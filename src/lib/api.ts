// API wrapper for Tauri commands

import { invoke } from '@tauri-apps/api/core';
import type { SkillInfo, TargetInfo, SyncResult, StatsInfo, DiscoveredSkillInfo, ImportSelectionInfo, ImportResultInfo } from './types';

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
