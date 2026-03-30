/**
 * Layout Migration System
 * Task 6.3: Automatic schema migration for saved layouts
 */

interface LayoutVersion {
  version: number;
  timestamp: number;
}

interface MigrationResult {
  success: boolean;
  migratedVersion?: number;
  errors?: string[];
}

/**
 * Current layout schema version
 */
export const CURRENT_LAYOUT_VERSION = 1;

/**
 * Layout migration functions
 * Each migration transforms a layout from version N to N+1
 */
const migrations: Array<(layout: any) => any> = [
  // Migration 0 -> 1: Initial version, add version field
  (layout: any) => {
    if (!layout.version) {
      return {
        version: 1,
        ...layout,
        migrated: true,
        migratedAt: Date.now(),
      };
    }
    return layout;
  },

  // Future migrations go here
  // Migration 1 -> 2: Example
  // (layout: any) => {
  //   return {
  //     ...layout,
  //     version: 2,
  //     // Add new fields or transform existing ones
  //   };
  // },
];

/**
 * Check if a layout needs migration
 */
export function needsMigration(layoutJson: string): boolean {
  try {
    const layout = JSON.parse(layoutJson);
    const version = layout.version || 0;
    return version < CURRENT_LAYOUT_VERSION;
  } catch (err) {
    console.error('Failed to parse layout for migration check:', err);
    return false;
  }
}

/**
 * Get layout version
 */
export function getLayoutVersion(layoutJson: string): number {
  try {
    const layout = JSON.parse(layoutJson);
    return layout.version || 0;
  } catch (err) {
    console.error('Failed to get layout version:', err);
    return 0;
  }
}

/**
 * Migrate layout to current version
 */
export function migrateLayout(layoutJson: string): MigrationResult {
  try {
    let layout = JSON.parse(layoutJson);
    const startVersion = layout.version || 0;
    const errors: string[] = [];

    // Apply migrations sequentially
    for (let i = startVersion; i < CURRENT_LAYOUT_VERSION; i++) {
      const migration = migrations[i];
      if (!migration) {
        errors.push(`Missing migration for version ${i} -> ${i + 1}`);
        continue;
      }

      try {
        layout = migration(layout);
        console.log(`[Migration] Applied migration ${i} -> ${i + 1}`);
      } catch (err) {
        const errorMsg = `Migration ${i} -> ${i + 1} failed: ${err}`;
        console.error(errorMsg);
        errors.push(errorMsg);
      }
    }

    // Verify final version
    if (layout.version !== CURRENT_LAYOUT_VERSION) {
      return {
        success: false,
        errors: [...errors, `Final version mismatch: expected ${CURRENT_LAYOUT_VERSION}, got ${layout.version}`],
      };
    }

    return {
      success: errors.length === 0,
      migratedVersion: CURRENT_LAYOUT_VERSION,
      errors: errors.length > 0 ? errors : undefined,
    };
  } catch (err) {
    return {
      success: false,
      errors: [`Failed to migrate layout: ${err}`],
    };
  }
}

/**
 * Validate layout structure
 */
export function validateLayout(layoutJson: string): { valid: boolean; errors?: string[] } {
  const errors: string[] = [];

  try {
    const layout = JSON.parse(layoutJson);

    // Check required fields
    if (!layout.panels) {
      errors.push('Missing required field: panels');
    } else if (!Array.isArray(layout.panels)) {
      errors.push('Field "panels" must be an array');
    }

    if (typeof layout.total_height !== 'number') {
      errors.push('Missing or invalid field: total_height');
    }

    // Check version
    if (layout.version !== CURRENT_LAYOUT_VERSION) {
      errors.push(`Version mismatch: expected ${CURRENT_LAYOUT_VERSION}, got ${layout.version}`);
    }

    // Validate panels
    if (Array.isArray(layout.panels)) {
      layout.panels.forEach((panel: any, index: number) => {
        if (!panel.id) {
          errors.push(`Panel ${index}: missing id`);
        }
        if (!panel.panel_type) {
          errors.push(`Panel ${index}: missing panel_type`);
        }
        if (typeof panel.computed_height !== 'number') {
          errors.push(`Panel ${index}: invalid computed_height`);
        }
      });
    }

    return {
      valid: errors.length === 0,
      errors: errors.length > 0 ? errors : undefined,
    };
  } catch (err) {
    return {
      valid: false,
      errors: [`Failed to parse layout: ${err}`],
    };
  }
}

/**
 * Automatically migrate and validate a layout
 * Returns migrated layout JSON or null if migration failed
 */
export function autoMigrateLayout(layoutJson: string): string | null {
  try {
    // Check if migration is needed
    if (!needsMigration(layoutJson)) {
      // Validate current version
      const validation = validateLayout(layoutJson);
      if (validation.valid) {
        return layoutJson;
      } else {
        console.warn('Layout validation failed:', validation.errors);
        return null;
      }
    }

    // Migrate
    const result = migrateLayout(layoutJson);

    if (!result.success) {
      console.error('Layout migration failed:', result.errors);
      return null;
    }

    if (result.errors && result.errors.length > 0) {
      console.warn('Layout migration completed with warnings:', result.errors);
    }

    // Parse and re-serialize to get clean JSON
    const migrated = JSON.parse(layoutJson);
    const migratedJson = JSON.stringify(migrated);

    // Validate migrated layout
    const validation = validateLayout(migratedJson);
    if (!validation.valid) {
      console.error('Migrated layout validation failed:', validation.errors);
      return null;
    }

    console.log(`[Migration] Successfully migrated layout from v${getLayoutVersion(layoutJson)} to v${CURRENT_LAYOUT_VERSION}`);
    return migratedJson;
  } catch (err) {
    console.error('Auto-migration failed:', err);
    return null;
  }
}

/**
 * Add version info to a new layout
 */
export function addVersionInfo(layoutJson: string): string {
  try {
    const layout = JSON.parse(layoutJson);

    // Add version if not present
    if (!layout.version) {
      layout.version = CURRENT_LAYOUT_VERSION;
      layout.createdAt = Date.now();
    }

    return JSON.stringify(layout);
  } catch (err) {
    console.error('Failed to add version info:', err);
    return layoutJson;
  }
}

/**
 * Check if all saved layouts need migration
 * Returns count of layouts needing migration
 */
export async function checkSavedLayoutsMigration(): Promise<{
  total: number;
  needsMigration: number;
  layouts: Array<{ id: string; name: string; currentVersion: number }>;
}> {
  try {
    const { layoutStorage } = await import('./layout-storage');
    const savedLayouts = await layoutStorage.getAllLayouts();

    const layoutsNeedingMigration = savedLayouts.filter((saved) =>
      needsMigration(saved.layout)
    );

    return {
      total: savedLayouts.length,
      needsMigration: layoutsNeedingMigration.length,
      layouts: layoutsNeedingMigration.map((saved) => ({
        id: saved.id,
        name: saved.name,
        currentVersion: getLayoutVersion(saved.layout),
      })),
    };
  } catch (err) {
    console.error('Failed to check saved layouts:', err);
    return {
      total: 0,
      needsMigration: 0,
      layouts: [],
    };
  }
}

/**
 * Migrate all saved layouts
 */
export async function migrateAllSavedLayouts(): Promise<{
  success: number;
  failed: number;
  errors: Array<{ id: string; name: string; error: string }>;
}> {
  try {
    const { layoutStorage } = await import('./layout-storage');
    const savedLayouts = await layoutStorage.getAllLayouts();

    let successCount = 0;
    let failedCount = 0;
    const errors: Array<{ id: string; name: string; error: string }> = [];

    for (const saved of savedLayouts) {
      if (!needsMigration(saved.layout)) {
        continue;
      }

      const migrated = autoMigrateLayout(saved.layout);

      if (migrated) {
        try {
          await layoutStorage.saveLayout(saved.name, migrated, saved.isDefault);
          successCount++;
        } catch (err) {
          failedCount++;
          errors.push({
            id: saved.id,
            name: saved.name,
            error: `Save failed: ${err}`,
          });
        }
      } else {
        failedCount++;
        errors.push({
          id: saved.id,
          name: saved.name,
          error: 'Migration failed',
        });
      }
    }

    return {
      success: successCount,
      failed: failedCount,
      errors,
    };
  } catch (err) {
    console.error('Failed to migrate all layouts:', err);
    return {
      success: 0,
      failed: 0,
      errors: [{ id: '', name: '', error: `${err}` }],
    };
  }
}
