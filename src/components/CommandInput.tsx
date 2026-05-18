import { useState, useCallback, KeyboardEvent } from 'react';
import { useGameStore } from '../stores/gameStore';
import { submitDirective, getAdvisorSuggestion } from '../utils/api';

const MAX_CHARS = 500;

const CommandInput: React.FC = () => {
  const [command, setCommand] = useState('');
  const [useAdvisor, setUseAdvisor] = useState(false);

  const isLoading = useGameStore((s) => s.isLoading);
  const playerCountry = useGameStore((s) => s.playerCountry);
  const settings = useGameStore((s) => s.settings);
  const setLoading = useGameStore((s) => s.setLoading);
  const addChatMessage = useGameStore((s) => s.addChatMessage);
  const setAdvisorOutput = useGameStore((s) => s.setAdvisorOutput);
  const setShowAdvisor = useGameStore((s) => s.setShowAdvisor);
  const gameState = useGameStore((s) => s.gameState);

  const isdisabled = isLoading || !playerCountry || !gameState?.active;

  const handleSubmit = useCallback(async () => {
    const trimmed = command.trim();
    if (!trimmed || isdisabled) return;

    addChatMessage('you', trimmed);
    setCommand('');
    setLoading(true);

    try {
      const response = await submitDirective(trimmed, useAdvisor);
      if (response) {
        addChatMessage('world', response);
      }
    } catch (err: any) {
      addChatMessage('system', `Error: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [command, isdisabled, useAdvisor, addChatMessage, setLoading]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSubmit();
      }
    },
    [handleSubmit]
  );

  async function handleAdvisor() {
    const trimmed = command.trim();
    if (!trimmed || !playerCountry) return;

    setLoading(true);
    try {
      const suggestion = await getAdvisorSuggestion(trimmed);
      setAdvisorOutput(suggestion);
      setShowAdvisor(true);
    } catch (err: any) {
      addChatMessage('system', `Advisor error: ${err}`);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="command-input">
      {!playerCountry && (
        <div className="command-input-blocked">
          Select a country to issue directives.
        </div>
      )}
      <div className="command-input-area">
        <textarea
          className="command-input-field"
          value={command}
          onChange={(e) => setCommand(e.target.value.slice(0, MAX_CHARS))}
          onKeyDown={handleKeyDown}
          placeholder={
            playerCountry
              ? `Directive for ${playerCountry.name}... (Enter to send)`
              : 'Select a country first...'
          }
          disabled={isdisabled}
          rows={2}
          maxLength={MAX_CHARS}
        />
        <div className="command-input-actions">
          <div className="command-input-left">
            <label className="advisor-toggle">
              <input
                type="checkbox"
                checked={useAdvisor}
                onChange={(e) => setUseAdvisor(e.target.checked)}
                disabled={isdisabled || !settings.advisorEnabled}
              />
              <span>Advisor</span>
            </label>
            <span className="command-char-count">
              {command.length}/{MAX_CHARS}
            </span>
          </div>
          <div className="command-input-right">
            <button
              className="btn btn--advisor"
              onClick={handleAdvisor}
              disabled={isdisabled || !settings.advisorEnabled || !command.trim()}
              title="Get advisor suggestion"
            >
              ?
            </button>
            <button
              className="btn btn--send"
              onClick={handleSubmit}
              disabled={isdisabled || !command.trim()}
            >
              {isLoading ? 'Sending...' : 'Send'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CommandInput;
