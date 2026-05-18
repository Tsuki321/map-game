import React, { useEffect, useState } from 'react';
import { useGameStore } from '../stores/gameStore';
import {
  configureLlm,
  saveGame,
  loadGame,
  listSaves,
} from '../utils/api';

interface SettingsPanelProps {
  onClose: () => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({ onClose }) => {
  const { settings, setSettings, setGameState } = useGameStore();
  const [provider, setProvider] = useState(
    settings.llmConfig?.provider_type || 'openai'
  );
  const [apiKey, setApiKey] = useState(settings.llmConfig?.api_key || '');
  const [model, setModel] = useState(
    settings.llmConfig?.model || 'gpt-4'
  );
  const [endpoint, setEndpoint] = useState(
    settings.llmConfig?.endpoint || ''
  );
  const [saving, setSaving] = useState(false);
  const [saveSlot, setSaveSlot] = useState('save1');
  const [savesList, setSavesList] = useState<string[]>([]);
  const [loadingList, setLoadingList] = useState(false);

  useEffect(() => {
    refreshSaves();
  }, []);

  const refreshSaves = async () => {
    setLoadingList(true);
    try {
      const saves = await listSaves();
      setSavesList(saves);
    } catch {
      setSavesList([]);
    }
    setLoadingList(false);
  };

  const handleSaveConfig = async () => {
    setSaving(true);
    try {
      await configureLlm(
        provider,
        apiKey,
        model,
        provider === 'ollama' ? endpoint || 'http://localhost:11434' : endpoint
      );
      setSettings({
        llmConfig: {
          provider_type: provider,
          api_key: apiKey,
          model,
          endpoint:
            provider === 'ollama'
              ? endpoint || 'http://localhost:11434'
              : endpoint,
        },
      });
    } catch (e) {
      console.error('Failed to configure LLM:', e);
    }
    setSaving(false);
  };

  const handleSave = async () => {
    try {
      await saveGame(saveSlot);
      refreshSaves();
    } catch (e) {
      console.error('Failed to save:', e);
    }
  };

  const handleLoad = async (slot: string) => {
    try {
      const state = await loadGame(slot);
      setGameState(state);
      onClose();
    } catch (e) {
      console.error('Failed to load:', e);
    }
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-card" onClick={(e) => e.stopPropagation()}>
        <h2>Settings</h2>

        <div className="form-group">
          <label>LLM Provider</label>
          <select value={provider} onChange={(e) => setProvider(e.target.value)}>
            <option value="openai">OpenAI (or compatible API)</option>
            <option value="ollama">Ollama (Local)</option>
          </select>
        </div>

        {provider === 'openai' && (
          <div className="form-group">
            <label>API Key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
            />
          </div>
        )}

        <div className="form-group">
          <label>Model</label>
          <input
            type="text"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            placeholder={provider === 'ollama' ? 'llama3' : 'gpt-4'}
          />
        </div>

        <div className="form-group">
          <label>
            Endpoint {provider === 'openai' ? '(optional custom)' : '(Ollama URL)'}
          </label>
          <input
            type="text"
            value={endpoint}
            onChange={(e) => setEndpoint(e.target.value)}
            placeholder={
              provider === 'ollama'
                ? 'http://localhost:11434'
                : 'https://api.openai.com/v1'
            }
          />
        </div>

        <button onClick={handleSaveConfig} disabled={saving} className="btn-primary">
          {saving ? 'Saving...' : 'Save LLM Config'}
        </button>

        <div className="form-group" style={{ marginTop: 8 }}>
          <label>
            <input
              type="checkbox"
              checked={settings.advisorEnabled}
              onChange={(e) =>
                setSettings({ advisorEnabled: e.target.checked })
              }
            />{' '}
            Enable Strategic Advisor
          </label>
        </div>

        <hr style={{ border: '1px solid #30363d', margin: '16px 0' }} />

        <h3>Save / Load Game</h3>
        <div className="form-group">
          <label>Save Slot Name</label>
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              type="text"
              value={saveSlot}
              onChange={(e) => setSaveSlot(e.target.value)}
              placeholder="my_save"
              style={{ flex: 1 }}
            />
            <button onClick={handleSave}>Save</button>
          </div>
        </div>

        <div className="form-group">
          <label>Saved Games</label>
          {loadingList ? (
            <span className="loading-spinner" />
          ) : savesList.length === 0 ? (
            <p style={{ color: '#8b949e' }}>No saves found.</p>
          ) : (
            <div style={{ maxHeight: 150, overflowY: 'auto' }}>
              {savesList.map((save) => {
                const [slot] = save.split(' | ');
                return (
                  <div
                    key={slot}
                    className="country-option"
                    onClick={() => handleLoad(slot)}
                  >
                    {save}
                  </div>
                );
              })}
            </div>
          )}
        </div>

        <button onClick={onClose} style={{ marginTop: 16 }}>
          Close
        </button>
      </div>
    </div>
  );
};

export default SettingsPanel;
