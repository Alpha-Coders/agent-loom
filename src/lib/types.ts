// Types that mirror the Rust backend structs
// See https://agentskills.io/specification for field definitions

export interface SkillInfo {
  // Required fields (per spec)
  name: string;
  folder_name: string;
  description: string;

  // Optional spec fields
  license: string | null;
  compatibility: string | null;
  metadata: Record<string, string>;
  allowed_tools: string | null;

  // Legacy fields (not in spec, kept for backward compatibility)
  tags: string[];
  version: string | null;
  author: string | null;

  // Internal fields
  path: string;
  validation_status: 'unknown' | 'valid' | 'invalid';
  validation_errors: string[];
}

export interface SyncStatus {
  is_synced: boolean;
  missing_skills: string[];
  extra_items: string[];
  broken_links: string[];
}

export interface TargetInfo {
  id: string;
  name: string;
  skills_path: string;
  auto_detected: boolean;
  enabled: boolean;
  exists: boolean;
  sync_status: SyncStatus | null;
}

export interface SyncResult {
  target_id: string;
  target_name: string;
  created: string[];
  removed: string[];
  unchanged: string[];
  errors: SyncError[];
}

export interface SyncError {
  skill: string | null;
  message: string;
}

export interface StatsInfo {
  total_skills: number;
  valid_skills: number;
  invalid_skills: number;
  total_targets: number;
  enabled_targets: number;
  is_watching: boolean;
}

export interface DiscoveredSkillInfo {
  name: string;
  description: string;
  source_path: string;
  source_target: string;
  has_conflict: boolean;
  existing_description: string | null;
}

export interface ImportSelectionInfo {
  name: string;
  source_path: string;
  resolution: 'import' | 'skip' | 'overwrite';
}

export interface ImportResultInfo {
  imported: string[];
  skipped: string[];
  errors: [string, string][];
  synced_to: number;
}

// === Folder Import Types ===

export interface ScannedSkillInfo {
  name: string;
  description: string;
  source_path: string;
  needs_fixes: boolean;
  fixes_preview: string[];
  has_conflict: boolean;
  existing_description: string | null;
}

export interface FolderImportSelectionInfo {
  name: string;
  source_path: string;
  apply_fixes: boolean;
  resolution: 'import' | 'skip' | 'overwrite';
}

// === Migration Types ===

export interface MigrationResult {
  migrated: boolean;
  skills_count: number;
  skill_names: string[];
  from_path: string | null;
  to_path: string | null;
  errors: string[];
}
