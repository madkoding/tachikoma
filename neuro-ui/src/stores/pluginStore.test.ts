import { describe, it, expect, beforeEach } from 'vitest';
import { usePluginStore, PLUGIN_CATALOG } from './pluginStore';

describe('usePluginStore', () => {
  beforeEach(() => {
    usePluginStore.setState({ installed: [] });
  });

  it('starts empty', () => {
    expect(usePluginStore.getState().installed).toEqual([]);
    expect(PLUGIN_CATALOG.length).toBeGreaterThan(0);
  });

  it('installs a plugin once', () => {
    const s = usePluginStore.getState();
    s.install('weather');
    s.install('weather');
    expect(usePluginStore.getState().installed).toEqual(['weather']);
  });

  it('uninstalls a plugin', () => {
    const s = usePluginStore.getState();
    s.install('weather');
    s.uninstall('weather');
    expect(usePluginStore.getState().installed).toEqual([]);
  });

  it('isInstalled reflects state', () => {
    const s = usePluginStore.getState();
    expect(s.isInstalled('weather')).toBe(false);
    s.install('weather');
    expect(s.isInstalled('weather')).toBe(true);
  });

  it('catalog ids are unique', () => {
    const ids = PLUGIN_CATALOG.map((p) => p.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
