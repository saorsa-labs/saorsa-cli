//! Rendering and layout utilities
//!
//! This module provides layout calculation for the main application
//! frame and pane area distribution.
//!
//! # Overview
//!
//! The renderer module handles two main concerns:
//!
//! 1. **Application Layout**: Dividing the terminal into tab bar, content,
//!    and status bar regions via [`AppLayout`].
//!
//! 2. **Pane Layout**: Calculating areas for panes within the content region
//!    based on a [`PaneLayout`] tree via [`calculate_pane_areas`].
//!
//! # Example
//!
//! ```ignore
//! use saorsa_ui::renderer::{AppLayout, calculate_pane_areas};
//! use saorsa_cli_core::{PaneLayout, PaneNode};
//!
//! // Calculate main application layout
//! let app_layout = AppLayout::new(frame.area());
//!
//! // Calculate pane areas within the content region
//! let pane_layout = PaneLayout {
//!     root: PaneNode::vsplit(30, vec![
//!         PaneNode::leaf(0),
//!         PaneNode::leaf(1),
//!     ]),
//! };
//! let pane_areas = calculate_pane_areas(&pane_layout, app_layout.content);
//!
//! // Render each pane in its calculated area
//! for (pane_id, area) in pane_areas {
//!     // render pane content...
//! }
//! ```

use ratatui::prelude::*;
use saorsa_cli_core::{PaneId, PaneLayout, PaneNode, Split};

/// Main application layout areas
///
/// Divides the terminal into tab bar, content, and status bar regions.
/// This provides a consistent layout structure for the main application.
///
/// # Layout Structure
///
/// ```text
/// +---------------------------------+
/// | Tab Bar (1 line)                |
/// +---------------------------------+
/// |                                 |
/// | Content Area                    |
/// | (remaining space)               |
/// |                                 |
/// +---------------------------------+
/// | Status Bar (1 line)             |
/// +---------------------------------+
/// ```
///
/// # Example
///
/// ```
/// use saorsa_ui::renderer::AppLayout;
/// use ratatui::prelude::Rect;
///
/// let area = Rect::new(0, 0, 80, 24);
/// let layout = AppLayout::new(area);
///
/// assert_eq!(layout.tab_bar.height, 1);
/// assert_eq!(layout.status_bar.height, 1);
/// assert_eq!(layout.content.height, 22);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AppLayout {
    /// Area for the tab bar (top)
    pub tab_bar: Rect,
    /// Area for the main content (middle)
    pub content: Rect,
    /// Area for the status bar (bottom)
    pub status_bar: Rect,
}

impl AppLayout {
    /// Calculate layout areas from the total terminal area
    ///
    /// Layout structure:
    /// - Tab bar: 1 line at top
    /// - Content: everything in between
    /// - Status bar: 1 line at bottom
    ///
    /// For terminals with height less than 3, the layout degrades gracefully
    /// by giving priority to content and reducing or eliminating decorations.
    ///
    /// # Arguments
    ///
    /// * `area` - The total available terminal area
    ///
    /// # Returns
    ///
    /// An `AppLayout` with calculated regions for each component.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::renderer::AppLayout;
    /// use ratatui::prelude::Rect;
    ///
    /// let area = Rect::new(0, 0, 80, 24);
    /// let layout = AppLayout::new(area);
    ///
    /// assert_eq!(layout.tab_bar, Rect::new(0, 0, 80, 1));
    /// assert_eq!(layout.content, Rect::new(0, 1, 80, 22));
    /// assert_eq!(layout.status_bar, Rect::new(0, 23, 80, 1));
    /// ```
    #[must_use]
    pub fn new(area: Rect) -> Self {
        if area.height < 3 {
            // Minimal space - give everything to content
            return AppLayout {
                tab_bar: Rect::new(area.x, area.y, area.width, 1.min(area.height)),
                content: Rect::new(area.x, area.y, area.width, area.height),
                status_bar: Rect::default(),
            };
        }

        let tab_bar = Rect::new(area.x, area.y, area.width, 1);
        let status_bar = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        let content = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(2),
        );

        AppLayout {
            tab_bar,
            content,
            status_bar,
        }
    }

    /// Returns the total width of the layout
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::renderer::AppLayout;
    /// use ratatui::prelude::Rect;
    ///
    /// let layout = AppLayout::new(Rect::new(0, 0, 80, 24));
    /// assert_eq!(layout.width(), 80);
    /// ```
    #[must_use]
    pub fn width(&self) -> u16 {
        self.content.width
    }

    /// Returns the total height of the layout
    ///
    /// This is the sum of all three regions.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::renderer::AppLayout;
    /// use ratatui::prelude::Rect;
    ///
    /// let layout = AppLayout::new(Rect::new(0, 0, 80, 24));
    /// assert_eq!(layout.height(), 24);
    /// ```
    #[must_use]
    pub fn height(&self) -> u16 {
        self.tab_bar.height + self.content.height + self.status_bar.height
    }
}

/// Calculates areas for panes in a layout
///
/// Given a [`PaneLayout`] tree and a content area, this function traverses
/// the layout tree and calculates the screen [`Rect`] for each leaf pane.
///
/// # Arguments
///
/// * `layout` - The pane layout tree to calculate areas for
/// * `area` - The total available area for panes
///
/// # Returns
///
/// A vector of (PaneId, Rect) pairs for each leaf pane in the layout.
/// The order matches a depth-first traversal of the layout tree.
///
/// # Example
///
/// ```
/// use saorsa_ui::renderer::calculate_pane_areas;
/// use saorsa_cli_core::{PaneLayout, PaneNode};
/// use ratatui::prelude::Rect;
///
/// // Single pane - gets full area
/// let layout = PaneLayout::single(0);
/// let area = Rect::new(0, 0, 80, 24);
/// let areas = calculate_pane_areas(&layout, area);
///
/// assert_eq!(areas.len(), 1);
/// assert_eq!(areas[0], (0, area));
/// ```
///
/// ```
/// use saorsa_ui::renderer::calculate_pane_areas;
/// use saorsa_cli_core::{PaneLayout, PaneNode};
/// use ratatui::prelude::Rect;
///
/// // Vertical split - 30% left, 70% right
/// let layout = PaneLayout {
///     root: PaneNode::vsplit(30, vec![
///         PaneNode::leaf(0),
///         PaneNode::leaf(1),
///     ]),
/// };
/// let area = Rect::new(0, 0, 100, 24);
/// let areas = calculate_pane_areas(&layout, area);
///
/// assert_eq!(areas.len(), 2);
/// // Left pane should be approximately 30% width
/// assert!(areas[0].1.width >= 25 && areas[0].1.width <= 35);
/// ```
#[must_use]
pub fn calculate_pane_areas(layout: &PaneLayout, area: Rect) -> Vec<(PaneId, Rect)> {
    let mut result = Vec::new();
    calculate_node_areas(&layout.root, area, &mut result);
    result
}

/// Recursively calculates areas for nodes in the layout tree
fn calculate_node_areas(node: &PaneNode, area: Rect, result: &mut Vec<(PaneId, Rect)>) {
    match node {
        PaneNode::Leaf(id) => {
            result.push((*id, area));
        }
        PaneNode::Split {
            direction,
            children,
        } => {
            if children.is_empty() {
                return;
            }

            let areas = split_area(area, direction, children.len());
            for (child, child_area) in children.iter().zip(areas) {
                calculate_node_areas(child, child_area, result);
            }
        }
    }
}

/// Splits an area according to a direction and child count
fn split_area(area: Rect, direction: &Split, count: usize) -> Vec<Rect> {
    if count == 0 || area.width == 0 || area.height == 0 {
        return vec![];
    }

    let ratio = direction.ratio();

    match direction {
        Split::Horizontal(_) => split_horizontal(area, ratio, count),
        Split::Vertical(_) => split_vertical(area, ratio, count),
    }
}

/// Splits area horizontally (top/bottom)
fn split_horizontal(area: Rect, ratio: u16, count: usize) -> Vec<Rect> {
    if count == 1 {
        return vec![area];
    }

    let first_height = calculate_first_dimension(area.height, ratio);

    let first = Rect::new(area.x, area.y, area.width, first_height);
    let second = Rect::new(
        area.x,
        area.y + first_height,
        area.width,
        area.height.saturating_sub(first_height),
    );

    if count == 2 {
        vec![first, second]
    } else {
        // For more than 2, divide equally after the first
        let mut result = vec![first];
        let remaining_height = second.height;
        let each_height = remaining_height / (count - 1).max(1) as u16;

        for i in 0..(count - 1) {
            let y = second.y + (i as u16 * each_height);
            let h = if i == count - 2 {
                second.y + remaining_height - y
            } else {
                each_height
            };
            result.push(Rect::new(area.x, y, area.width, h));
        }
        result
    }
}

/// Splits area vertically (left/right)
fn split_vertical(area: Rect, ratio: u16, count: usize) -> Vec<Rect> {
    if count == 1 {
        return vec![area];
    }

    let first_width = calculate_first_dimension(area.width, ratio);

    let first = Rect::new(area.x, area.y, first_width, area.height);
    let second = Rect::new(
        area.x + first_width,
        area.y,
        area.width.saturating_sub(first_width),
        area.height,
    );

    if count == 2 {
        vec![first, second]
    } else {
        let mut result = vec![first];
        let remaining_width = second.width;
        let each_width = remaining_width / (count - 1).max(1) as u16;

        for i in 0..(count - 1) {
            let x = second.x + (i as u16 * each_width);
            let w = if i == count - 2 {
                second.x + remaining_width - x
            } else {
                each_width
            };
            result.push(Rect::new(x, area.y, w, area.height));
        }
        result
    }
}

/// Calculates the first dimension based on ratio
fn calculate_first_dimension(total: u16, ratio: u16) -> u16 {
    let first = (u32::from(total) * u32::from(ratio) / 100) as u16;
    first.max(1).min(total.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_layout_normal() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = AppLayout::new(area);

        assert_eq!(layout.tab_bar, Rect::new(0, 0, 80, 1));
        assert_eq!(layout.content, Rect::new(0, 1, 80, 22));
        assert_eq!(layout.status_bar, Rect::new(0, 23, 80, 1));
    }

    #[test]
    fn test_app_layout_minimal() {
        let area = Rect::new(0, 0, 80, 2);
        let layout = AppLayout::new(area);
        // Should still work with minimal height
        assert!(layout.tab_bar.height > 0 || layout.content.height > 0);
    }

    #[test]
    fn test_app_layout_zero_height() {
        let area = Rect::new(0, 0, 80, 0);
        let layout = AppLayout::new(area);
        // Should handle zero height gracefully
        assert_eq!(layout.tab_bar.height, 0);
    }

    #[test]
    fn test_app_layout_with_offset() {
        let area = Rect::new(10, 5, 60, 20);
        let layout = AppLayout::new(area);

        assert_eq!(layout.tab_bar.x, 10);
        assert_eq!(layout.tab_bar.y, 5);
        assert_eq!(layout.content.x, 10);
        assert_eq!(layout.content.y, 6);
        assert_eq!(layout.status_bar.y, 24); // 5 + 20 - 1
    }

    #[test]
    fn test_app_layout_width_height() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = AppLayout::new(area);

        assert_eq!(layout.width(), 80);
        assert_eq!(layout.height(), 24);
    }

    #[test]
    fn test_app_layout_default() {
        let layout = AppLayout::default();
        assert_eq!(layout.tab_bar, Rect::default());
        assert_eq!(layout.content, Rect::default());
        assert_eq!(layout.status_bar, Rect::default());
    }

    #[test]
    fn test_pane_areas_single() {
        let layout = PaneLayout::single(0);
        let area = Rect::new(0, 0, 80, 24);
        let areas = calculate_pane_areas(&layout, area);

        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0], (0, area));
    }

    #[test]
    fn test_pane_areas_vsplit() {
        let layout = PaneLayout {
            root: PaneNode::vsplit(30, vec![PaneNode::leaf(0), PaneNode::leaf(1)]),
        };
        let area = Rect::new(0, 0, 100, 24);
        let areas = calculate_pane_areas(&layout, area);

        assert_eq!(areas.len(), 2);
        // First pane should be ~30%
        assert!(areas[0].1.width >= 25 && areas[0].1.width <= 35);
        // Second pane should be the rest
        assert!(areas[1].1.width >= 65);
        // Total width should equal area width
        assert_eq!(areas[0].1.width + areas[1].1.width, area.width);
    }

    #[test]
    fn test_pane_areas_hsplit() {
        let layout = PaneLayout {
            root: PaneNode::hsplit(50, vec![PaneNode::leaf(0), PaneNode::leaf(1)]),
        };
        let area = Rect::new(0, 0, 80, 20);
        let areas = calculate_pane_areas(&layout, area);

        assert_eq!(areas.len(), 2);
        // Both panes should be ~50% height
        assert!(areas[0].1.height >= 8 && areas[0].1.height <= 12);
        assert!(areas[1].1.height >= 8 && areas[1].1.height <= 12);
        // Total height should equal area height
        assert_eq!(areas[0].1.height + areas[1].1.height, area.height);
    }

    #[test]
    fn test_pane_areas_nested() {
        // Layout: left pane (30%) | right side with top/bottom split (70%)
        let layout = PaneLayout {
            root: PaneNode::vsplit(
                30,
                vec![
                    PaneNode::leaf(0),
                    PaneNode::hsplit(50, vec![PaneNode::leaf(1), PaneNode::leaf(2)]),
                ],
            ),
        };
        let area = Rect::new(0, 0, 100, 20);
        let areas = calculate_pane_areas(&layout, area);

        assert_eq!(areas.len(), 3);
        // Verify pane IDs are in depth-first order
        assert_eq!(areas[0].0, 0);
        assert_eq!(areas[1].0, 1);
        assert_eq!(areas[2].0, 2);
    }

    #[test]
    fn test_pane_areas_empty_children() {
        let layout = PaneLayout {
            root: PaneNode::Split {
                direction: Split::Vertical(50),
                children: vec![],
            },
        };
        let area = Rect::new(0, 0, 80, 24);
        let areas = calculate_pane_areas(&layout, area);

        assert!(areas.is_empty());
    }

    #[test]
    fn test_pane_areas_zero_dimension() {
        let layout = PaneLayout::single(0);
        let area = Rect::new(0, 0, 0, 0);
        let areas = calculate_pane_areas(&layout, area);

        // Should return the leaf with zero-sized area
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].0, 0);
    }

    #[test]
    fn test_pane_areas_single_child_split() {
        let layout = PaneLayout {
            root: PaneNode::vsplit(50, vec![PaneNode::leaf(0)]),
        };
        let area = Rect::new(0, 0, 80, 24);
        let areas = calculate_pane_areas(&layout, area);

        // Single child should get the full area
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0], (0, area));
    }

    #[test]
    fn test_pane_areas_three_way_split() {
        let layout = PaneLayout {
            root: PaneNode::vsplit(
                33,
                vec![PaneNode::leaf(0), PaneNode::leaf(1), PaneNode::leaf(2)],
            ),
        };
        let area = Rect::new(0, 0, 90, 24);
        let areas = calculate_pane_areas(&layout, area);

        assert_eq!(areas.len(), 3);
        // First pane should be ~33%
        assert!(areas[0].1.width >= 25 && areas[0].1.width <= 35);
    }

    #[test]
    fn test_split_area_empty_count() {
        let area = Rect::new(0, 0, 80, 24);
        let result = split_area(area, &Split::Vertical(50), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_area_zero_width() {
        let area = Rect::new(0, 0, 0, 24);
        let result = split_area(area, &Split::Vertical(50), 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_area_zero_height() {
        let area = Rect::new(0, 0, 80, 0);
        let result = split_area(area, &Split::Horizontal(50), 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_calculate_first_dimension() {
        // 100 * 30% = 30
        assert_eq!(calculate_first_dimension(100, 30), 30);

        // 80 * 50% = 40
        assert_eq!(calculate_first_dimension(80, 50), 40);

        // 100 * 0% should be at least 1
        assert_eq!(calculate_first_dimension(100, 0), 1);

        // 100 * 100% should leave at least 1 for second pane
        assert_eq!(calculate_first_dimension(100, 100), 99);
    }

    #[test]
    fn test_app_layout_clone() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = AppLayout::new(area);
        let cloned = layout;

        assert_eq!(layout, cloned);
    }

    #[test]
    fn test_app_layout_debug() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = AppLayout::new(area);
        let debug_str = format!("{:?}", layout);

        assert!(debug_str.contains("AppLayout"));
        assert!(debug_str.contains("tab_bar"));
        assert!(debug_str.contains("content"));
        assert!(debug_str.contains("status_bar"));
    }
}
