import { useEffect, useRef } from 'react';
import { useGameStore } from '../stores/gameStore';

const ROLE_LABELS: Record<string, string> = {
  system: 'System',
  you: 'You',
  advisor: 'Advisor',
  world: 'World',
  player: 'You',
};

const Chatbox: React.FC = () => {
  const chatHistory = useGameStore((s) => s.chatHistory);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [chatHistory]);

  return (
    <div className="chatbox">
      {chatHistory.length === 0 && (
        <div className="chatbox-empty">
          <div className="chatbox-empty-icon">&#9879;</div>
          <p>No messages yet.</p>
          <p className="chatbox-empty-hint">
            Select a country and issue your first directive.
          </p>
        </div>
      )}

      <div className="chatbox-messages">
        {chatHistory.map((msg, i) => (
          <div key={i} className={`chat-message chat-message--${msg.role}`}>
            <span className={`chat-badge chat-badge--${msg.role}`}>
              {ROLE_LABELS[msg.role] || msg.role}
            </span>
            <div className="chat-content">
              {msg.content.split('\n').map((line, li) => (
                <span key={li}>
                  {line}
                  {li < msg.content.split('\n').length - 1 && <br />}
                </span>
              ))}
            </div>
            <span className="chat-time">
              {new Date(msg.timestamp).toLocaleTimeString([], {
                hour: '2-digit',
                minute: '2-digit',
              })}
            </span>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
};

export default Chatbox;
