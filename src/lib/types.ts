// Types that mirror the Rust backend structs

export interface SkillInfo {
  name: string;
  description: string;
  tags: string[];
  version: string | null;
  author: string | null;
  path: string;
  validation_status: 'unknown' | 'valid' | 'invalid';
  validation_errors: string[];
}

export interface TargetInfo {
  id: string;
  name: string;
  skills_path: string;
  auto_detected: boolean;
  enabled: boolean;
  exists: boolean;
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
