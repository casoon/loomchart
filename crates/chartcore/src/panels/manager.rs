//! Panel manager for layout and coordination

use std::collections::HashMap;

use super::panel::{Panel, PanelConfig, PanelId};

/// Manages all panels and their layout
pub struct PanelManager {
    /// Ordered list of panels
    panels: Vec<Panel>,
    /// Quick lookup: id -> index
    panel_map: HashMap<PanelId, usize>,

    // Layout state
    /// Total available height in pixels
    total_height: u32,
    /// Height of separator between panels
    separator_height: u32,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            panel_map: HashMap::new(),
            total_height: 0,
            separator_height: 1,
        }
    }

    /// Add a panel at specific position (or end if None)
    pub fn add_panel(&mut self, config: PanelConfig, position: Option<usize>) -> PanelId {
        let id = config.id;
        let panel = Panel::new(config);

        let insert_pos = position.unwrap_or(self.panels.len()).min(self.panels.len());
        self.panels.insert(insert_pos, panel);

        self.rebuild_index();
        self.recalculate_layout();

        id
    }

    /// Remove panel by ID
    pub fn remove_panel(&mut self, id: PanelId) -> Option<PanelConfig> {
        if let Some(&index) = self.panel_map.get(&id) {
            let panel = self.panels.remove(index);
            self.rebuild_index();
            self.recalculate_layout();
            Some(panel.config)
        } else {
            None
        }
    }

    /// Move panel to new position
    pub fn move_panel(&mut self, id: PanelId, new_index: usize) -> bool {
        if let Some(&current_index) = self.panel_map.get(&id) {
            if current_index == new_index || new_index >= self.panels.len() {
                return false;
            }

            let panel = self.panels.remove(current_index);
            self.panels.insert(new_index, panel);

            self.rebuild_index();
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Swap two panels
    pub fn swap_panels(&mut self, id1: PanelId, id2: PanelId) -> bool {
        if let (Some(&idx1), Some(&idx2)) = (self.panel_map.get(&id1), self.panel_map.get(&id2)) {
            self.panels.swap(idx1, idx2);
            self.rebuild_index();
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Set stretch factor for a panel (triggers relayout)
    pub fn set_stretch_factor(&mut self, id: PanelId, factor: f64) -> bool {
        if let Some(&index) = self.panel_map.get(&id) {
            self.panels[index].config.stretch_factor = factor.max(0.1);
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Set panel height directly (adjusts stretch factor to maintain proportion)
    pub fn set_panel_height(&mut self, id: PanelId, height: u32) -> bool {
        if let Some(&index) = self.panel_map.get(&id) {
            let current_height = self.panels[index].computed_height;
            if current_height == 0 {
                return false;
            }

            // Adjust stretch factor proportionally
            let ratio = height as f64 / current_height as f64;
            self.panels[index].config.stretch_factor *= ratio;
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Set total available height and recalculate
    pub fn set_total_height(&mut self, height: u32) {
        self.total_height = height;
        self.recalculate_layout();
    }

    /// Recalculate panel heights based on stretch factors
    pub fn recalculate_layout(&mut self) {
        if self.panels.is_empty() || self.total_height == 0 {
            return;
        }

        let num_separators = self.panels.len().saturating_sub(1);
        let separator_total = num_separators as u32 * self.separator_height;

        // Available height after separators
        let available = self.total_height.saturating_sub(separator_total);

        // Calculate total stretch from non-collapsed panels
        let total_stretch: f64 = self
            .panels
            .iter()
            .filter(|p| !p.config.collapsed)
            .map(|p| p.config.stretch_factor)
            .sum();

        if total_stretch == 0.0 {
            return;
        }

        // First pass: assign proportional heights
        let mut assigned_heights: Vec<u32> = self
            .panels
            .iter()
            .map(|p| {
                if p.config.collapsed {
                    0
                } else {
                    ((available as f64) * (p.config.stretch_factor / total_stretch)) as u32
                }
            })
            .collect();

        // Second pass: enforce min heights and redistribute
        let mut needs_redistribution = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        while needs_redistribution && iterations < MAX_ITERATIONS {
            needs_redistribution = false;
            iterations += 1;

            let mut excess = 0u32;
            let mut flexible_indices = Vec::new();

            for (i, panel) in self.panels.iter().enumerate() {
                if panel.config.collapsed {
                    continue;
                }

                if assigned_heights[i] < panel.config.min_height {
                    excess += panel.config.min_height - assigned_heights[i];
                    assigned_heights[i] = panel.config.min_height;
                    needs_redistribution = true;
                } else if assigned_heights[i] > panel.config.min_height {
                    flexible_indices.push(i);
                }
            }

            // Redistribute excess from flexible panels
            if needs_redistribution && !flexible_indices.is_empty() {
                let reduction_per_panel = excess / flexible_indices.len() as u32;
                for &i in &flexible_indices {
                    assigned_heights[i] = assigned_heights[i].saturating_sub(reduction_per_panel);
                }
            }
        }

        // Third pass: assign computed values and y_offsets
        let mut current_y = 0u32;
        for (i, panel) in self.panels.iter_mut().enumerate() {
            if panel.config.collapsed {
                panel.computed_height = 0;
                panel.y_offset = current_y;
            } else {
                panel.computed_height = assigned_heights[i];
                panel.y_offset = current_y;
                current_y += assigned_heights[i] + self.separator_height;
            }
        }
    }

    /// Get panel by ID
    pub fn get_panel(&self, id: PanelId) -> Option<&Panel> {
        self.panel_map.get(&id).map(|&idx| &self.panels[idx])
    }

    /// Get mutable panel by ID
    pub fn get_panel_mut(&mut self, id: PanelId) -> Option<&mut Panel> {
        if let Some(&idx) = self.panel_map.get(&id) {
            Some(&mut self.panels[idx])
        } else {
            None
        }
    }

    /// Get panel at Y coordinate
    pub fn panel_at_y(&self, y: u32) -> Option<&Panel> {
        self.panels
            .iter()
            .find(|p| !p.config.collapsed && y >= p.y_offset && y < p.y_offset + p.computed_height)
    }

    /// Get all panels in order
    pub fn panels(&self) -> &[Panel] {
        &self.panels
    }

    /// Get all panels mutably
    pub fn panels_mut(&mut self) -> &mut [Panel] {
        &mut self.panels
    }

    /// Get total computed height (including separators)
    pub fn computed_total_height(&self) -> u32 {
        if self.panels.is_empty() {
            return 0;
        }

        let last = self.panels.last().unwrap();
        last.y_offset + last.computed_height
    }

    /// Serialize layout to JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "panels": self.panels.iter().map(|p| &p.config).collect::<Vec<_>>(),
            "total_height": self.total_height,
        })
    }

    /// Restore layout from JSON
    pub fn from_json(&mut self, json: &str) -> Result<(), String> {
        let data: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

        let configs: Vec<PanelConfig> = serde_json::from_value(data["panels"].clone())
            .map_err(|e| format!("Invalid panel configs: {}", e))?;

        self.panels.clear();
        self.panel_map.clear();

        for config in configs {
            self.add_panel(config, None);
        }

        if let Some(height) = data["total_height"].as_u64() {
            self.set_total_height(height as u32);
        }

        Ok(())
    }

    fn rebuild_index(&mut self) {
        self.panel_map.clear();
        for (idx, panel) in self.panels.iter().enumerate() {
            self.panel_map.insert(panel.id(), idx);
        }
    }

    /// Reorder panels by moving from_index to to_index (Phase 3 Task 5.2)
    pub fn reorder_panels(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.panels.len() || to_index >= self.panels.len() {
            return false;
        }

        if from_index == to_index {
            return true; // No-op
        }

        // Remove panel from old position
        let panel = self.panels.remove(from_index);

        // Insert at new position
        self.panels.insert(to_index, panel);

        // Rebuild panel_map with new indices
        self.panel_map.clear();
        for (idx, panel) in self.panels.iter().enumerate() {
            self.panel_map.insert(panel.id(), idx);
        }

        // Recalculate layout
        self.recalculate_layout();

        true
    }

    /// Collapse/minimize a panel (Task 5.3)
    pub fn collapse_panel(&mut self, id: PanelId) -> bool {
        if let Some(&index) = self.panel_map.get(&id) {
            self.panels[index].config.collapsed = true;
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Expand/restore a panel (Task 5.3)
    pub fn expand_panel(&mut self, id: PanelId) -> bool {
        if let Some(&index) = self.panel_map.get(&id) {
            self.panels[index].config.collapsed = false;
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Maximize a panel (collapse all others) (Task 5.3)
    pub fn maximize_panel(&mut self, id: PanelId) -> bool {
        if !self.panel_map.contains_key(&id) {
            return false;
        }

        // Collapse all panels except the target
        for panel in &mut self.panels {
            panel.config.collapsed = panel.id() != id;
        }

        self.recalculate_layout();
        true
    }

    /// Restore all panels (expand all collapsed panels) (Task 5.3)
    pub fn restore_all_panels(&mut self) -> bool {
        let mut changed = false;
        for panel in &mut self.panels {
            if panel.config.collapsed {
                panel.config.collapsed = false;
                changed = true;
            }
        }

        if changed {
            self.recalculate_layout();
        }

        changed
    }

    /// Check if a panel is collapsed
    pub fn is_panel_collapsed(&self, id: PanelId) -> Option<bool> {
        self.panel_map
            .get(&id)
            .map(|&idx| self.panels[idx].config.collapsed)
    }

    /// Check if a panel is maximized (all others are collapsed)
    pub fn is_panel_maximized(&self, id: PanelId) -> Option<bool> {
        if !self.panel_map.contains_key(&id) {
            return None;
        }

        // A panel is maximized if it's not collapsed and all others are
        let target_collapsed = self.is_panel_collapsed(id)?;
        if target_collapsed {
            return Some(false);
        }

        let all_others_collapsed = self
            .panels
            .iter()
            .filter(|p| p.id() != id)
            .all(|p| p.config.collapsed);

        Some(all_others_collapsed)
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_layout() {
        let mut manager = PanelManager::new();
        manager.set_total_height(500);

        // Add 3 panels: Chart (3x), RSI (1x), MACD (1x)
        let chart = PanelConfig::new_chart();
        let chart_id = chart.id;
        manager.add_panel(chart, None);

        let rsi = PanelConfig::new_indicator("rsi14", serde_json::json!({"period": 14}));
        manager.add_panel(rsi, None);

        let macd = PanelConfig::new_indicator("macd", serde_json::json!({}));
        manager.add_panel(macd, None);

        // Total stretch: 3 + 1 + 1 = 5
        // Chart should get 3/5 = 60% of available height
        // Available: 500 - 2 separators = 498
        // Chart: 298px, RSI: 100px, MACD: 100px

        let panels = manager.panels();
        assert_eq!(panels.len(), 3);

        let chart_panel = &panels[0];
        assert_eq!(chart_panel.y_offset, 0);
        assert!(chart_panel.computed_height > 200); // Should be ~298

        let rsi_panel = &panels[1];
        assert!(rsi_panel.y_offset > 200);
        assert!(rsi_panel.computed_height >= 80); // Min height enforced
    }

    #[test]
    fn test_panel_removal() {
        let mut manager = PanelManager::new();
        manager.set_total_height(400);

        let chart = PanelConfig::new_chart();
        let chart_id = chart.id;
        manager.add_panel(chart, None);

        let rsi = PanelConfig::new_indicator("rsi14", serde_json::json!({}));
        let rsi_id = rsi.id;
        manager.add_panel(rsi, None);

        assert_eq!(manager.panels().len(), 2);

        manager.remove_panel(rsi_id);
        assert_eq!(manager.panels().len(), 1);
        assert_eq!(manager.get_panel(chart_id).unwrap().id(), chart_id);
    }
}
