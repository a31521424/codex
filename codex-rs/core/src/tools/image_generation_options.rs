use codex_tools::ImageGenerationAction;
use codex_tools::ImageGenerationBackground;
use codex_tools::ImageGenerationInputImageMask;
use codex_tools::ImageGenerationModeration;
use codex_tools::ImageGenerationOutputFormat;
use codex_tools::ImageGenerationQuality;
use codex_tools::ImageGenerationToolOptions;
use serde_json::Value;

const ACTION_ENV: &str = "CODEX_IMAGE_GENERATION_ACTION";
const SIZE_ENV: &str = "CODEX_IMAGE_GENERATION_SIZE";
const QUALITY_ENV: &str = "CODEX_IMAGE_GENERATION_QUALITY";
const OUTPUT_FORMAT_ENV: &str = "CODEX_IMAGE_GENERATION_OUTPUT_FORMAT";
const OUTPUT_COMPRESSION_ENV: &str = "CODEX_IMAGE_GENERATION_OUTPUT_COMPRESSION";
const BACKGROUND_ENV: &str = "CODEX_IMAGE_GENERATION_BACKGROUND";
const PARTIAL_IMAGES_ENV: &str = "CODEX_IMAGE_GENERATION_PARTIAL_IMAGES";
const INPUT_IMAGE_MASK_JSON_ENV: &str = "CODEX_IMAGE_GENERATION_INPUT_IMAGE_MASK_JSON";
const MODERATION_ENV: &str = "CODEX_IMAGE_GENERATION_MODERATION";

const MIN_IMAGE_PIXELS: u64 = 655_360;
const MAX_IMAGE_PIXELS: u64 = 8_294_400;
const MAX_IMAGE_EDGE: u32 = 3_840;

pub(crate) fn resolve_image_generation_tool_options_from_env()
-> Result<ImageGenerationToolOptions, String> {
    resolve_image_generation_tool_options(|name| std::env::var(name).ok())
}

fn resolve_image_generation_tool_options(
    get_var: impl Fn(&str) -> Option<String>,
) -> Result<ImageGenerationToolOptions, String> {
    let mut options = ImageGenerationToolOptions::default();

    if let Some(value) = read_env_value(&get_var, ACTION_ENV)? {
        options.action = Some(parse_action(&value)?);
    }
    if let Some(value) = read_env_value(&get_var, SIZE_ENV)? {
        options.size = Some(parse_size(&value)?);
    }
    if let Some(value) = read_env_value(&get_var, QUALITY_ENV)? {
        options.quality = Some(parse_quality(&value)?);
    }
    if let Some(value) = read_env_value(&get_var, OUTPUT_FORMAT_ENV)? {
        options.output_format = parse_output_format(&value)?;
    }
    if let Some(value) = read_env_value(&get_var, OUTPUT_COMPRESSION_ENV)? {
        options.output_compression =
            Some(parse_bounded_u8(&value, OUTPUT_COMPRESSION_ENV, 0, 100)?);
    }
    if let Some(value) = read_env_value(&get_var, BACKGROUND_ENV)? {
        options.background = Some(parse_background(&value)?);
    }
    if let Some(value) = read_env_value(&get_var, PARTIAL_IMAGES_ENV)? {
        options.partial_images = Some(parse_bounded_u8(&value, PARTIAL_IMAGES_ENV, 0, 3)?);
    }
    if let Some(value) = read_env_value(&get_var, INPUT_IMAGE_MASK_JSON_ENV)? {
        options.input_image_mask = Some(parse_input_image_mask(&value)?);
    }
    if let Some(value) = read_env_value(&get_var, MODERATION_ENV)? {
        options.moderation = Some(parse_moderation(&value)?);
    }

    if options.output_compression.is_some()
        && matches!(options.output_format, ImageGenerationOutputFormat::Png)
    {
        return Err(format!(
            "{OUTPUT_COMPRESSION_ENV} is only supported when {OUTPUT_FORMAT_ENV} is `jpeg` or `webp`; current output_format is `png`"
        ));
    }

    Ok(options)
}

fn read_env_value(
    get_var: &impl Fn(&str) -> Option<String>,
    name: &'static str,
) -> Result<Option<String>, String> {
    let Some(value) = get_var(name) else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{name} must not be empty"));
    }
    Ok(Some(value.to_string()))
}

fn parse_action(value: &str) -> Result<ImageGenerationAction, String> {
    match value {
        "auto" => Ok(ImageGenerationAction::Auto),
        "generate" => Ok(ImageGenerationAction::Generate),
        "edit" => Ok(ImageGenerationAction::Edit),
        _ => Err(format!(
            "{ACTION_ENV} must be one of `auto`, `generate`, or `edit`; got `{value}`"
        )),
    }
}

fn parse_quality(value: &str) -> Result<ImageGenerationQuality, String> {
    match value {
        "low" => Ok(ImageGenerationQuality::Low),
        "medium" => Ok(ImageGenerationQuality::Medium),
        "high" => Ok(ImageGenerationQuality::High),
        "auto" => Ok(ImageGenerationQuality::Auto),
        _ => Err(format!(
            "{QUALITY_ENV} must be one of `low`, `medium`, `high`, or `auto`; got `{value}`"
        )),
    }
}

fn parse_output_format(value: &str) -> Result<ImageGenerationOutputFormat, String> {
    match value {
        "png" => Ok(ImageGenerationOutputFormat::Png),
        "jpeg" => Ok(ImageGenerationOutputFormat::Jpeg),
        "webp" => Ok(ImageGenerationOutputFormat::Webp),
        _ => Err(format!(
            "{OUTPUT_FORMAT_ENV} must be one of `png`, `jpeg`, or `webp`; got `{value}`"
        )),
    }
}

fn parse_background(value: &str) -> Result<ImageGenerationBackground, String> {
    match value {
        "auto" => Ok(ImageGenerationBackground::Auto),
        "opaque" => Ok(ImageGenerationBackground::Opaque),
        "transparent" => Err(format!(
            "{BACKGROUND_ENV}=transparent is not supported by gpt-image-2 through the Responses image_generation tool"
        )),
        _ => Err(format!(
            "{BACKGROUND_ENV} must be `auto` or `opaque`; got `{value}`"
        )),
    }
}

fn parse_moderation(value: &str) -> Result<ImageGenerationModeration, String> {
    match value {
        "auto" => Ok(ImageGenerationModeration::Auto),
        "low" => Ok(ImageGenerationModeration::Low),
        _ => Err(format!(
            "{MODERATION_ENV} must be `auto` or `low`; got `{value}`"
        )),
    }
}

fn parse_bounded_u8(value: &str, env_name: &'static str, min: u8, max: u8) -> Result<u8, String> {
    let parsed = value
        .parse::<u8>()
        .map_err(|_| format!("{env_name} must be an integer from {min} to {max}; got `{value}`"))?;
    if !(min..=max).contains(&parsed) {
        return Err(format!(
            "{env_name} must be an integer from {min} to {max}; got `{value}`"
        ));
    }
    Ok(parsed)
}

fn parse_size(value: &str) -> Result<String, String> {
    if value == "auto" {
        return Ok(value.to_string());
    }

    let parts = value.split('x').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(invalid_size_message(value));
    }
    let width = parts[0]
        .parse::<u32>()
        .map_err(|_| invalid_size_message(value))?;
    let height = parts[1]
        .parse::<u32>()
        .map_err(|_| invalid_size_message(value))?;
    if width == 0 || height == 0 {
        return Err(invalid_size_message(value));
    }
    if width > MAX_IMAGE_EDGE || height > MAX_IMAGE_EDGE {
        return Err(invalid_size_message(value));
    }
    if width % 16 != 0 || height % 16 != 0 {
        return Err(invalid_size_message(value));
    }

    let long_edge = width.max(height);
    let short_edge = width.min(height);
    if long_edge > short_edge.saturating_mul(3) {
        return Err(invalid_size_message(value));
    }

    let pixels = u64::from(width) * u64::from(height);
    if !(MIN_IMAGE_PIXELS..=MAX_IMAGE_PIXELS).contains(&pixels) {
        return Err(invalid_size_message(value));
    }

    Ok(value.to_string())
}

fn invalid_size_message(value: &str) -> String {
    format!(
        "{SIZE_ENV} must be `auto` or WIDTHxHEIGHT with max edge <= {MAX_IMAGE_EDGE}, both edges multiples of 16, long:short ratio <= 3:1, and total pixels {MIN_IMAGE_PIXELS}..={MAX_IMAGE_PIXELS}; got `{value}`"
    )
}

fn parse_input_image_mask(value: &str) -> Result<ImageGenerationInputImageMask, String> {
    let value = serde_json::from_str::<Value>(value)
        .map_err(|err| format!("{INPUT_IMAGE_MASK_JSON_ENV} must be a JSON object: {err}"))?;
    let Some(object) = value.as_object() else {
        return Err(format!("{INPUT_IMAGE_MASK_JSON_ENV} must be a JSON object"));
    };

    for key in object.keys() {
        if key != "file_id" && key != "image_url" {
            return Err(format!(
                "{INPUT_IMAGE_MASK_JSON_ENV} may only contain `file_id` or `image_url`; got unexpected key `{key}`"
            ));
        }
    }

    let file_id = optional_non_empty_string_field(object, "file_id")?;
    let image_url = optional_non_empty_string_field(object, "image_url")?;
    if file_id.is_some() == image_url.is_some() {
        return Err(format!(
            "{INPUT_IMAGE_MASK_JSON_ENV} must contain exactly one of `file_id` or `image_url`"
        ));
    }

    Ok(ImageGenerationInputImageMask { file_id, image_url })
}

fn optional_non_empty_string_field(
    object: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<Option<String>, String> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(format!(
            "{INPUT_IMAGE_MASK_JSON_ENV}.{field} must be a string"
        ));
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(format!(
            "{INPUT_IMAGE_MASK_JSON_ENV}.{field} must not be empty"
        ));
    }
    Ok(Some(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    fn resolve(values: &[(&str, &str)]) -> Result<ImageGenerationToolOptions, String> {
        let values = values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect::<BTreeMap<_, _>>();
        resolve_image_generation_tool_options(|name| values.get(name).cloned())
    }

    #[test]
    fn resolver_accepts_valid_all_field_config() {
        assert_eq!(
            resolve(&[
                (ACTION_ENV, "edit"),
                (SIZE_ENV, "1536x1024"),
                (QUALITY_ENV, "high"),
                (OUTPUT_FORMAT_ENV, "webp"),
                (OUTPUT_COMPRESSION_ENV, "80"),
                (BACKGROUND_ENV, "opaque"),
                (PARTIAL_IMAGES_ENV, "3"),
                (
                    INPUT_IMAGE_MASK_JSON_ENV,
                    r#"{"image_url":"https://example.com/mask.png"}"#
                ),
                (MODERATION_ENV, "low"),
            ])
            .expect("valid options"),
            ImageGenerationToolOptions {
                output_format: ImageGenerationOutputFormat::Webp,
                action: Some(ImageGenerationAction::Edit),
                size: Some("1536x1024".to_string()),
                quality: Some(ImageGenerationQuality::High),
                output_compression: Some(80),
                background: Some(ImageGenerationBackground::Opaque),
                partial_images: Some(3),
                input_image_mask: Some(ImageGenerationInputImageMask {
                    file_id: None,
                    image_url: Some("https://example.com/mask.png".to_string()),
                }),
                moderation: Some(ImageGenerationModeration::Low),
            }
        );
    }

    #[test]
    fn resolver_rejects_invalid_size() {
        let err = resolve(&[(SIZE_ENV, "512x512")]).expect_err("invalid size");

        assert!(err.contains(SIZE_ENV));
        assert!(err.contains("655360"));
    }

    #[test]
    fn resolver_rejects_transparent_background() {
        let err = resolve(&[(BACKGROUND_ENV, "transparent")]).expect_err("invalid background");

        assert!(err.contains("transparent"));
        assert!(err.contains("gpt-image-2"));
    }

    #[test]
    fn resolver_rejects_png_output_compression() {
        let err =
            resolve(&[(OUTPUT_COMPRESSION_ENV, "80")]).expect_err("invalid output compression");

        assert!(err.contains(OUTPUT_COMPRESSION_ENV));
        assert!(err.contains("jpeg"));
        assert!(err.contains("webp"));
    }

    #[test]
    fn resolver_passes_through_moderation() {
        assert_eq!(
            resolve(&[(MODERATION_ENV, "low")])
                .expect("valid moderation")
                .moderation,
            Some(ImageGenerationModeration::Low)
        );
    }
}
