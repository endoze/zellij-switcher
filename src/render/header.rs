use zellij_tile::prelude::*;

use crate::config::PluginConfig;

/// Calculates the x-coordinate to horizontally center a title of the given
/// length within the available columns.
pub fn calculate_header_x(title_len: usize, cols: usize) -> usize {
  cols.saturating_sub(title_len) / 2
}

/// Renders the centered header title at the top of the plugin pane.
pub fn render_header(config: &PluginConfig, cols: usize) {
  let x = calculate_header_x(config.header_title.len(), cols);
  let header = Text::new(&config.header_title).color_range(config.header_text_color, ..);

  print_text_with_coordinates(header, x, 0, None, None);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn centers_title_in_columns() {
    assert_eq!(calculate_header_x(10, 80), 35);
  }

  #[test]
  fn centers_title_exactly() {
    assert_eq!(calculate_header_x(4, 20), 8);
  }

  #[test]
  fn handles_title_wider_than_cols() {
    assert_eq!(calculate_header_x(100, 50), 0);
  }

  #[test]
  fn handles_zero_cols() {
    assert_eq!(calculate_header_x(10, 0), 0);
  }

  #[test]
  fn render_header_does_not_panic() {
    let config = PluginConfig::default();
    render_header(&config, 80);
  }

  #[test]
  fn render_header_narrow_does_not_panic() {
    let config = PluginConfig::default();
    render_header(&config, 0);
  }
}
