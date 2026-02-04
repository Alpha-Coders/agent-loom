// API wrapper for Tauri commands

import { invoke } from '@tauri-apps/api/core';
import type { SkillInfo, TargetInfo, SyncResult, StatsInfo, DiscoveredSkillInfo, ImportSelectionInfo, ImportResultInfo, ScannedSkillInfo, FolderImportSelectionInfo, MigrationResult } from './types';

export async function getSkills(): Promise<SkillInfo[]> {
  return invoke<SkillInfo[]>('get_skills');
}

export async function getTargets(): Promise<TargetInfo[]> {
  return invoke<TargetInfo[]>('get_targets');
}

export async function syncAll(): Promise<SyncResult[]> {
  return invoke<SyncResult[]>('sync_all');
}

export async function createSkill(name: string, description: string): Promise<SkillInfo> {
  return invoke<SkillInfo>('create_skill', { name, description });
}

export async function validateSkill(name: string): Promise<SkillInfo> {
  return invoke<SkillInfo>('validate_skill', { name });
}

export async function validateAll(): Promise<SkillInfo[]> {
  return invoke<SkillInfo[]>('validate_all');
}

export async function refreshSkills(): Promise<SkillInfo[]> {
  return invoke<SkillInfo[]>('refresh_skills');
}

export async function searchSkills(query: string): Promise<string[]> {
  return invoke<string[]>('search_skills', { query });
}

export async function deleteSkill(name: string): Promise<void> {
  return invoke<void>('delete_skill', { name });
}

export async function renameSkill(oldName: string, newName: string): Promise<SkillInfo> {
  return invoke<SkillInfo>('rename_skill', { oldName, newName });
}

export async function getStats(): Promise<StatsInfo> {
  return invoke<StatsInfo>('get_stats');
}

export async function getSkillContent(name: string): Promise<string> {
  return invoke<string>('get_skill_content', { name });
}

export async function saveSkillContent(name: string, content: string): Promise<SkillInfo> {
  return invoke<SkillInfo>('save_skill_content', { name, content });
}

export async function discoverImportableSkills(): Promise<DiscoveredSkillInfo[]> {
  return invoke<DiscoveredSkillInfo[]>('discover_importable_skills');
}

export async function importSkills(selections: ImportSelectionInfo[]): Promise<ImportResultInfo> {
  return invoke<ImportResultInfo>('import_skills', { selections });
}

export async function importAllSkills(): Promise<ImportResultInfo> {
  return invoke<ImportResultInfo>('import_all_skills');
}

export async function isFileMergeAvailable(): Promise<boolean> {
  return invoke<boolean>('is_filemerge_available');
}

export async function launchFileMerge(existing: string, incoming: string): Promise<void> {
  return invoke<void>('launch_filemerge', { existing, incoming });
}

// Target management

export async function toggleTarget(targetId: string): Promise<boolean> {
  return invoke<boolean>('toggle_target', { targetId });
}

export async function setTargetEnabled(targetId: string, enabled: boolean): Promise<void> {
  return invoke<void>('set_target_enabled', { targetId, enabled });
}

export async function addFolderTarget(path: string): Promise<TargetInfo> {
  return invoke<TargetInfo>('add_folder_target', { path });
}

// Skill fixing

export async function fixSkill(name: string): Promise<SkillInfo> {
  return invoke<SkillInfo>('fix_skill', { name });
}

export async function fixAllSkills(): Promise<[string, string[]][]> {
  return invoke<[string, string[]][]>('fix_all_skills');
}

// Menu state

export async function setSaveMenuEnabled(enabled: boolean): Promise<void> {
  return invoke<void>('set_save_menu_enabled', { enabled });
}

// Folder import

export async function scanFolderForSkills(path: string): Promise<ScannedSkillInfo[]> {
  return invoke<ScannedSkillInfo[]>('scan_folder_for_skills', { path });
}

export async function importFromFolder(selections: FolderImportSelectionInfo[]): Promise<ImportResultInfo> {
  return invoke<ImportResultInfo>('import_from_folder', { selections });
}

// Finder integration

export async function revealInFinder(path: string): Promise<void> {
  return invoke<void>('reveal_in_finder', { path });
}

// Migration

export async function checkAndMigrate(): Promise<MigrationResult> {
  return invoke<MigrationResult>('check_and_migrate');
}

export async function hasLegacySkills(): Promise<boolean> {
  return invoke<boolean>('has_legacy_skills');
}
