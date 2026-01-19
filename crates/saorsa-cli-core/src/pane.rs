//! Pane layout types for split view management.
//!
//! This module provides types for managing split pane layouts within tabs.
//! Panes can be arranged in a tree structure with horizontal and vertical
//! splits at various ratios.

/// Unique identifier for a pane.
///
/// Each pane in a layout has a unique numeric identifier used
/// for focus management and content mapping.
pub type PaneId = u32;

/// Direction and ratio for splitting a pane.
///
/// Splits divide a pane into two regions with a configurable ratio.
/// The ratio specifies the percentage (0-100) allocated to the first pane.
///
/// # Examples
///
/// ```
/// use saorsa_cli_core::Split;
///
/// // 60% top, 40% bottom
/// let hsplit = Split::Horizontal(60);
/// assert!(hsplit.is_horizontal());
/// assert_eq!(hsplit.ratio(), 60);
///
/// // 30% left, 70% right
/// let vsplit = Split::Vertical(30);
/// assert!(vsplit.is_vertical());
/// assert_eq!(vsplit.ratio(), 30);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    /// Horizontal split (top/bottom) with percentage for first pane.
    ///
    /// In a horizontal split, the first child appears on top and
    /// the second child appears on the bottom.
    Horizontal(u16),

    /// Vertical split (left/right) with percentage for first pane.
    ///
    /// In a vertical split, the first child appears on the left and
    /// the second child appears on the right.
    Vertical(u16),
}

impl Split {
    /// Returns true if this is a horizontal split.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::Split;
    ///
    /// assert!(Split::Horizontal(50).is_horizontal());
    /// assert!(!Split::Vertical(50).is_horizontal());
    /// ```
    #[must_use]
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Split::Horizontal(_))
    }

    /// Returns true if this is a vertical split.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::Split;
    ///
    /// assert!(Split::Vertical(50).is_vertical());
    /// assert!(!Split::Horizontal(50).is_vertical());
    /// ```
    #[must_use]
    pub fn is_vertical(&self) -> bool {
        matches!(self, Split::Vertical(_))
    }

    /// Returns the ratio (percentage) of the first pane.
    ///
    /// The ratio is a value from 0-100 representing the percentage
    /// of space allocated to the first child pane.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::Split;
    ///
    /// assert_eq!(Split::Horizontal(75).ratio(), 75);
    /// assert_eq!(Split::Vertical(25).ratio(), 25);
    /// ```
    #[must_use]
    pub fn ratio(&self) -> u16 {
        match self {
            Split::Horizontal(r) | Split::Vertical(r) => *r,
        }
    }
}

/// A node in the pane layout tree.
///
/// The layout is represented as a tree where leaf nodes contain pane IDs
/// and internal nodes represent splits with child subtrees.
///
/// # Examples
///
/// ```
/// use saorsa_cli_core::PaneNode;
///
/// // Single pane
/// let single = PaneNode::leaf(0);
/// assert_eq!(single.pane_ids(), vec![0]);
///
/// // Vertical split with two panes
/// let split = PaneNode::vsplit(50, vec![
///     PaneNode::leaf(1),
///     PaneNode::leaf(2),
/// ]);
/// assert_eq!(split.pane_ids(), vec![1, 2]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneNode {
    /// A leaf node containing a single pane.
    Leaf(PaneId),

    /// A split node containing child nodes.
    Split {
        /// The direction and ratio of the split.
        direction: Split,
        /// Child nodes (typically 2).
        children: Vec<PaneNode>,
    },
}

impl PaneNode {
    /// Creates a new leaf node.
    ///
    /// # Arguments
    ///
    /// * `id` - The pane identifier for this leaf
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// let leaf = PaneNode::leaf(42);
    /// assert_eq!(leaf.pane_ids(), vec![42]);
    /// ```
    #[must_use]
    pub fn leaf(id: PaneId) -> Self {
        PaneNode::Leaf(id)
    }

    /// Creates a new horizontal split.
    ///
    /// # Arguments
    ///
    /// * `ratio` - Percentage (0-100) for the top pane
    /// * `children` - Child nodes (typically 2)
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// let split = PaneNode::hsplit(60, vec![
    ///     PaneNode::leaf(0),
    ///     PaneNode::leaf(1),
    /// ]);
    /// ```
    #[must_use]
    pub fn hsplit(ratio: u16, children: Vec<PaneNode>) -> Self {
        PaneNode::Split {
            direction: Split::Horizontal(ratio),
            children,
        }
    }

    /// Creates a new vertical split.
    ///
    /// # Arguments
    ///
    /// * `ratio` - Percentage (0-100) for the left pane
    /// * `children` - Child nodes (typically 2)
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// let split = PaneNode::vsplit(30, vec![
    ///     PaneNode::leaf(0),
    ///     PaneNode::leaf(1),
    /// ]);
    /// ```
    #[must_use]
    pub fn vsplit(ratio: u16, children: Vec<PaneNode>) -> Self {
        PaneNode::Split {
            direction: Split::Vertical(ratio),
            children,
        }
    }

    /// Returns all pane IDs in this subtree.
    ///
    /// Traverses the tree and collects all pane identifiers from
    /// leaf nodes in depth-first order.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// let layout = PaneNode::hsplit(50, vec![
    ///     PaneNode::leaf(1),
    ///     PaneNode::vsplit(50, vec![
    ///         PaneNode::leaf(2),
    ///         PaneNode::leaf(3),
    ///     ]),
    /// ]);
    /// assert_eq!(layout.pane_ids(), vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn pane_ids(&self) -> Vec<PaneId> {
        match self {
            PaneNode::Leaf(id) => vec![*id],
            PaneNode::Split { children, .. } => {
                children.iter().flat_map(|c| c.pane_ids()).collect()
            }
        }
    }

    /// Returns the number of leaf panes in this subtree.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// let layout = PaneNode::hsplit(50, vec![
    ///     PaneNode::leaf(1),
    ///     PaneNode::leaf(2),
    /// ]);
    /// assert_eq!(layout.pane_count(), 2);
    /// ```
    #[must_use]
    pub fn pane_count(&self) -> usize {
        match self {
            PaneNode::Leaf(_) => 1,
            PaneNode::Split { children, .. } => children.iter().map(|c| c.pane_count()).sum(),
        }
    }

    /// Returns true if this node is a leaf.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// assert!(PaneNode::leaf(0).is_leaf());
    /// assert!(!PaneNode::hsplit(50, vec![PaneNode::leaf(0), PaneNode::leaf(1)]).is_leaf());
    /// ```
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(self, PaneNode::Leaf(_))
    }

    /// Returns true if this node is a split.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneNode;
    ///
    /// assert!(!PaneNode::leaf(0).is_split());
    /// assert!(PaneNode::hsplit(50, vec![PaneNode::leaf(0), PaneNode::leaf(1)]).is_split());
    /// ```
    #[must_use]
    pub fn is_split(&self) -> bool {
        matches!(self, PaneNode::Split { .. })
    }
}

/// Root layout structure for panes within a tab.
///
/// The `PaneLayout` wraps a [`PaneNode`] tree and provides methods
/// for querying and manipulating the overall layout.
///
/// # Examples
///
/// ```
/// use saorsa_cli_core::{PaneLayout, PaneNode};
///
/// // Create a simple single-pane layout
/// let layout = PaneLayout::single(0);
/// assert_eq!(layout.pane_ids(), vec![0]);
///
/// // Create a more complex layout
/// let complex = PaneLayout {
///     root: PaneNode::vsplit(30, vec![
///         PaneNode::leaf(0),
///         PaneNode::hsplit(50, vec![
///             PaneNode::leaf(1),
///             PaneNode::leaf(2),
///         ]),
///     ]),
/// };
/// assert_eq!(complex.pane_ids(), vec![0, 1, 2]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneLayout {
    /// The root node of the layout tree.
    pub root: PaneNode,
}

impl PaneLayout {
    /// Creates a new layout with a single pane.
    ///
    /// This is the simplest layout containing just one pane
    /// that fills the entire available area.
    ///
    /// # Arguments
    ///
    /// * `pane_id` - The identifier for the single pane
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneLayout;
    ///
    /// let layout = PaneLayout::single(0);
    /// assert_eq!(layout.pane_ids(), vec![0]);
    /// ```
    #[must_use]
    pub fn single(pane_id: PaneId) -> Self {
        PaneLayout {
            root: PaneNode::Leaf(pane_id),
        }
    }

    /// Returns all pane IDs in this layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneLayout;
    ///
    /// let layout = PaneLayout::single(42);
    /// assert_eq!(layout.pane_ids(), vec![42]);
    /// ```
    #[must_use]
    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.root.pane_ids()
    }

    /// Returns the number of panes in this layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::{PaneLayout, PaneNode};
    ///
    /// let single = PaneLayout::single(0);
    /// assert_eq!(single.pane_count(), 1);
    ///
    /// let split = PaneLayout {
    ///     root: PaneNode::hsplit(50, vec![
    ///         PaneNode::leaf(0),
    ///         PaneNode::leaf(1),
    ///     ]),
    /// };
    /// assert_eq!(split.pane_count(), 2);
    /// ```
    #[must_use]
    pub fn pane_count(&self) -> usize {
        self.root.pane_count()
    }

    /// Returns true if the layout contains the specified pane.
    ///
    /// # Arguments
    ///
    /// * `pane_id` - The pane identifier to search for
    ///
    /// # Examples
    ///
    /// ```
    /// use saorsa_cli_core::PaneLayout;
    ///
    /// let layout = PaneLayout::single(42);
    /// assert!(layout.contains(42));
    /// assert!(!layout.contains(0));
    /// ```
    #[must_use]
    pub fn contains(&self, pane_id: PaneId) -> bool {
        self.pane_ids().contains(&pane_id)
    }
}

impl Default for PaneLayout {
    /// Creates a default layout with a single pane (ID 0).
    fn default() -> Self {
        PaneLayout::single(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_horizontal() {
        let split = Split::Horizontal(60);
        assert!(split.is_horizontal());
        assert!(!split.is_vertical());
        assert_eq!(split.ratio(), 60);
    }

    #[test]
    fn test_split_vertical() {
        let split = Split::Vertical(40);
        assert!(split.is_vertical());
        assert!(!split.is_horizontal());
        assert_eq!(split.ratio(), 40);
    }

    #[test]
    fn test_split_clone() {
        let original = Split::Horizontal(50);
        let cloned = original;
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_pane_node_leaf() {
        let node = PaneNode::leaf(42);
        assert!(node.is_leaf());
        assert!(!node.is_split());
        assert_eq!(node.pane_ids(), vec![42]);
        assert_eq!(node.pane_count(), 1);
    }

    #[test]
    fn test_pane_node_hsplit() {
        let node = PaneNode::hsplit(60, vec![PaneNode::leaf(1), PaneNode::leaf(2)]);
        assert!(node.is_split());
        assert!(!node.is_leaf());
        assert_eq!(node.pane_ids(), vec![1, 2]);
        assert_eq!(node.pane_count(), 2);
    }

    #[test]
    fn test_pane_node_vsplit() {
        let node = PaneNode::vsplit(30, vec![PaneNode::leaf(3), PaneNode::leaf(4)]);
        assert!(node.is_split());
        assert_eq!(node.pane_ids(), vec![3, 4]);
    }

    #[test]
    fn test_pane_node_nested() {
        // Create: left | (top / bottom)
        let node = PaneNode::vsplit(
            30,
            vec![
                PaneNode::leaf(0),
                PaneNode::hsplit(50, vec![PaneNode::leaf(1), PaneNode::leaf(2)]),
            ],
        );
        assert_eq!(node.pane_ids(), vec![0, 1, 2]);
        assert_eq!(node.pane_count(), 3);
    }

    #[test]
    fn test_pane_node_deeply_nested() {
        let node = PaneNode::vsplit(
            50,
            vec![
                PaneNode::hsplit(50, vec![PaneNode::leaf(0), PaneNode::leaf(1)]),
                PaneNode::hsplit(50, vec![PaneNode::leaf(2), PaneNode::leaf(3)]),
            ],
        );
        assert_eq!(node.pane_ids(), vec![0, 1, 2, 3]);
        assert_eq!(node.pane_count(), 4);
    }

    #[test]
    fn test_pane_layout_single() {
        let layout = PaneLayout::single(5);
        assert_eq!(layout.pane_ids(), vec![5]);
        assert_eq!(layout.pane_count(), 1);
        assert!(layout.contains(5));
        assert!(!layout.contains(0));
    }

    #[test]
    fn test_pane_layout_default() {
        let layout = PaneLayout::default();
        assert_eq!(layout.pane_ids(), vec![0]);
        assert_eq!(layout.pane_count(), 1);
    }

    #[test]
    fn test_pane_layout_complex() {
        let layout = PaneLayout {
            root: PaneNode::vsplit(
                25,
                vec![
                    PaneNode::leaf(0),
                    PaneNode::hsplit(50, vec![PaneNode::leaf(1), PaneNode::leaf(2)]),
                ],
            ),
        };
        assert_eq!(layout.pane_ids(), vec![0, 1, 2]);
        assert_eq!(layout.pane_count(), 3);
        assert!(layout.contains(0));
        assert!(layout.contains(1));
        assert!(layout.contains(2));
        assert!(!layout.contains(3));
    }

    #[test]
    fn test_pane_layout_clone() {
        let original = PaneLayout::single(42);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_split_debug() {
        let split = Split::Horizontal(50);
        let debug_str = format!("{:?}", split);
        assert!(debug_str.contains("Horizontal"));
        assert!(debug_str.contains("50"));
    }

    #[test]
    fn test_pane_node_debug() {
        let node = PaneNode::leaf(42);
        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("Leaf"));
        assert!(debug_str.contains("42"));
    }
}
