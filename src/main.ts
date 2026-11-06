import './styles/base.css';
import { mount } from 'svelte';

// One bundle, two windows: the island and its settings.
const isSettings = location.hash.startsWith('#settings');
const target = document.getElementById('app')!;

if (isSettings) {
  const { default: SettingsApp } = await import('./SettingsApp.svelte');
  document.title = 'Holm · Settings';
  mount(SettingsApp, { target });
} else {
  const { default: IslandApp } = await import('./IslandApp.svelte');
  mount(IslandApp, { target });
}
