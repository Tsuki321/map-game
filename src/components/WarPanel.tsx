import { useState } from 'react';
import { useGameStore } from '../stores/gameStore';

interface WarInfo {
  war_id: string;
  name: string;
  opponent_name: string;
  status: string;
  fronts: WarFront[];
  casualties_ours: number;
  casualties_theirs: number;
  duration_turns: number;
}

interface WarFront {
  name: string;
  progress: number; // 0-100, 50 is stalemate, 100 is our victory
  description: string;
}

// Helper to derive wars from player country data
function deriveWars(): WarInfo[] {
  const state = useGameStore.getState();
  const player = state.playerCountry;
  if (!player || player.active_wars.length === 0) return [];
  // The actual war detail comes from the backend turn results.
  // For display we construct minimal info from what we have.
  return player.active_wars.map((wid) => ({
    war_id: wid,
    name: wid,
    opponent_name: wid,
    status: 'ongoing',
    fronts: [],
    casualties_ours: 0,
    casualties_theirs: 0,
    duration_turns: 0,
  }));
}

const WarPanel: React.FC = () => {
  const [expandedWars, setExpandedWars] = useState<Set<string>>(new Set());
  const playerCountry = useGameStore((s) => s.playerCountry);
  const lastTurnResult = useGameStore((s) => s.lastTurnResult);

  // Merge war info from playerCountry.active_wars and lastTurnResult.war_updates
  const wars = deriveWars();

  // Also check lastTurnResult for war updates
  const warUpdates = lastTurnResult?.war_updates || [];

  function toggleWar(warId: string) {
    setExpandedWars((prev) => {
      const next = new Set(prev);
      if (next.has(warId)) {
        next.delete(warId);
      } else {
        next.add(warId);
      }
      return next;
    });
  }

  if (!playerCountry || playerCountry.active_wars.length === 0) {
    // Show war updates from last turn if any
    if (warUpdates.length === 0) {
      return null;
    }
    return (
      <div className="war-panel">
        <div className="war-panel-header">War Updates</div>
        <div className="war-panel-content">
          {warUpdates.map((update: any, i: number) => (
            <div key={i} className="war-item">
              <div className="war-item-name">{update.name || update.war_id || `Update ${i + 1}`}</div>
              <div className="war-item-detail">{update.description || JSON.stringify(update)}</div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="war-panel">
      <div className="war-panel-header">
        Active Wars ({wars.length})
      </div>
      <div className="war-panel-content">
        {wars.map((war) => (
          <div key={war.war_id} className="war-item">
            <div
              className="war-item-header"
              onClick={() => toggleWar(war.war_id)}
            >
              <span className={`war-status war-status--${war.status}`}>
                {war.status}
              </span>
              <span className="war-item-name">{war.name}</span>
              <span className="war-item-toggle">
                {expandedWars.has(war.war_id) ? '▲' : '▼'}
              </span>
            </div>
            {expandedWars.has(war.war_id) && (
              <div className="war-item-details">
                {war.fronts.length > 0 ? (
                  war.fronts.map((front, fi) => (
                    <div key={fi} className="war-front">
                      <div className="war-front-name">{front.name}</div>
                      <div className="war-front-bar">
                        <div className="war-front-track">
                          <div
                            className="war-front-fill"
                            style={{ width: `${Math.min(front.progress, 100)}%` }}
                          />
                        </div>
                        <span>{front.progress}%</span>
                      </div>
                      <div className="war-front-desc">{front.description}</div>
                    </div>
                  ))
                ) : (
                  <div className="war-item-detail">
                    <p>Awaiting intelligence reports...</p>
                  </div>
                )}
                {(war.casualties_ours > 0 || war.casualties_theirs > 0) && (
                  <div className="war-casualties">
                    <span>Casualties: {war.casualties_ours.toLocaleString()} ours / {war.casualties_theirs.toLocaleString()} theirs</span>
                  </div>
                )}
                <div className="war-duration">
                  Duration: {war.duration_turns} turns
                </div>
              </div>
            )}
          </div>
        ))}

        {/* War updates from turn results */}
        {warUpdates.map((update: any, i: number) => (
          <div key={`wu-${i}`} className="war-item war-item--update">
            <div className="war-item-name">{update.name || update.war_id || `Update ${i + 1}`}</div>
            <div className="war-item-detail">{update.description || JSON.stringify(update)}</div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default WarPanel;
