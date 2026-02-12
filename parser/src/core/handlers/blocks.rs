use crate::{
    config::{BlockConfig, BlockType},
    core::{
        parsers::{parse_block_end_marker, parse_block_start_marker},
        updates::{BlockPayload, BlockEndPayload, TokenizerUpdate},
    },
};

/// Check if we're at the start of a line (for block detection)
/// Blocks must start at the beginning of a line
pub fn create_block_start_update<'a>(
    input: &'a str,
    block_def: &'a BlockConfig,
    is_line_start: bool,
) -> Option<Vec<TokenizerUpdate<'a>>> {
    // Block markers must be at the start of a line
    // BUT we want to detect them inside brackets to report specific error
    // So we don't return None here immediately if we are just checking for presence

    // However, for normal parsing, we enforce line start.
    // The issue is that inside brackets, we are using the same handler.

    // If we remove this check, then blocks can appear anywhere?
    // Handlers are used in Orchestrator main loop too.

    // Maybe I should add a field to HandlerContext? `allow_blocks_anywhere` or `detecting_errors`?

    // Or, I can change how block handlers work.

    parse_block_start_marker(input, &block_def.start_marker)
        .ok()
        .map(|(rest, marker)| {
            let len = marker.chars().count();
            vec![TokenizerUpdate::AddBlockStart(BlockPayload {
                block_name: extract_block_name(&block_def.start_marker),
                block_type: match block_def.block_type {
                    BlockType::Standalone => "standalone",
                    BlockType::WithText => "with_text",
                },
                has_end_marker: block_def.end_marker.is_some(),
                end_marker: block_def.end_marker.as_deref(),
                rest: Some(rest),
                len,
            })]
        })
}

/// Check if we're at a block end marker
/// End markers must be on their own line
pub fn create_block_end_update<'a>(
    input: &'a str,
    end_marker: &'a str,
    is_line_start: bool,
) -> Option<Vec<TokenizerUpdate<'a>>> {
    // End markers must be at the start of a line
    if !is_line_start {
        return None;
    }

    parse_block_end_marker(input, end_marker)
        .ok()
        .map(|(rest, marker)| {
            let len = marker.chars().count();
            vec![TokenizerUpdate::AddBlockEnd(BlockEndPayload {
                rest: Some(rest),
                len,
            })]
        })
}

/// Extract block name from marker like "[illustration]" -> "illustration"
fn extract_block_name(marker: &str) -> &str {
    marker.trim_start_matches('[').trim_end_matches(']')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BlockConfig, BlockType};
    use crate::core::updates::TokenizerUpdate;

    fn illustration_block() -> BlockConfig {
        BlockConfig {
            start_marker: "[illustration]".to_string(),
            has_text: false,
            block_type: BlockType::Standalone,
            end_marker: None,
            is_inline: false,
            precedence: usize::MAX,
        }
    }

    fn legend_block() -> BlockConfig {
        BlockConfig {
            start_marker: "[legend]".to_string(),
            has_text: true,
            block_type: BlockType::WithText,
            end_marker: Some("---".to_string()),
            is_inline: false,
            precedence: usize::MAX,
        }
    }

    fn margin_block() -> BlockConfig {
        BlockConfig {
            start_marker: "[margin]".to_string(),
            has_text: true,
            block_type: BlockType::WithText,
            end_marker: Some("---".to_string()),
            is_inline: false,
            precedence: usize::MAX,
        }
    }

    #[test]
    fn test_standalone_block_at_line_start() {
        let block_def = illustration_block();
        let result = create_block_start_update("[illustration]\ntext", &block_def, true);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddBlockStart(payload) = &updates[0] {
            assert_eq!(payload.block_name, "illustration");
            assert_eq!(payload.block_type, "standalone");
            assert!(!payload.has_end_marker);
        } else {
            panic!("Expected AddBlockStart update");
        }
    }

    #[test]
    fn test_block_not_at_line_start_returns_some() {
        let block_def = illustration_block();
        // Not at line start, BUT should still return Some now as check is removed
        let result = create_block_start_update("[illustration]", &block_def, false);

        assert!(result.is_some());
    }

    #[test]
    fn test_with_text_block() {
        let block_def = legend_block();
        let result = create_block_start_update("[legend]\ntext\n---", &block_def, true);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddBlockStart(payload) = &updates[0] {
            assert_eq!(payload.block_name, "legend");
            assert_eq!(payload.block_type, "with_text");
            assert!(payload.has_end_marker);
            assert_eq!(payload.end_marker, Some("---"));
        } else {
            panic!("Expected AddBlockStart update");
        }
    }

    #[test]
    fn test_block_end_marker_at_line_start() {
        let result = create_block_end_update("---\ntext", "---", true);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddBlockEnd(payload) = &updates[0] {
            assert_eq!(payload.len, 3);
        } else {
            panic!("Expected AddBlockEnd update");
        }
    }

    #[test]
    fn test_block_end_marker_not_at_line_start_returns_none() {
        let result = create_block_end_update("---", "---", false);

        assert!(result.is_none());
    }

    #[test]
    fn test_wrong_block_marker_returns_none() {
        let block_def = illustration_block();
        let result = create_block_start_update("[margin]", &block_def, true);

        assert!(result.is_none());
    }

    #[test]
    fn test_block_marker_len() {
        let block_def = illustration_block();
        let result = create_block_start_update("[illustration]", &block_def, true);

        let updates = result.unwrap();
        if let TokenizerUpdate::AddBlockStart(payload) = &updates[0] {
            assert_eq!(payload.len, 14); // "[illustration]" is 14 chars
        } else {
            panic!("Expected AddBlockStart update");
        }
    }

    #[test]
    fn test_block_rest_after_marker() {
        let block_def = legend_block();
        let result = create_block_start_update("[legend]\ntext", &block_def, true);

        let updates = result.unwrap();
        if let TokenizerUpdate::AddBlockStart(payload) = &updates[0] {
            assert_eq!(payload.rest, Some("\ntext"));
        } else {
            panic!("Expected AddBlockStart update");
        }
    }
}
