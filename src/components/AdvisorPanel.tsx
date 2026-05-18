import { useGameStore } from '../stores/gameStore';

interface AdvisorPanelProps {
  onAccept?: (text: string) => void;
}

const AdvisorPanel: React.FC<AdvisorPanelProps> = ({ onAccept }) => {
  const advisorOutput = useGameStore((s) => s.advisorOutput);
  const showAdvisor = useGameStore((s) => s.showAdvisor);
  const setShowAdvisor = useGameStore((s) => s.setShowAdvisor);
  const setAdvisorOutput = useGameStore((s) => s.setAdvisorOutput);

  if (!showAdvisor || !advisorOutput) return null;

  function handleAccept() {
    if (onAccept && advisorOutput) {
      onAccept(advisorOutput);
    }
    setShowAdvisor(false);
    setAdvisorOutput(null);
  }

  function handleReject() {
    setShowAdvisor(false);
    setAdvisorOutput(null);
  }

  return (
    <div className="advisor-panel">
      <div className="advisor-panel-header">
        <span className="advisor-panel-title">&#9888; Advisor Refinement</span>
      </div>
      <div className="advisor-panel-content">
        {advisorOutput.split('\n').map((line, i) => (
          <p key={i}>{line || '\u00A0'}</p>
        ))}
      </div>
      <div className="advisor-panel-actions">
        <button className="btn btn--accept" onClick={handleAccept}>
          Accept Directive
        </button>
        <button className="btn btn--reject" onClick={handleReject}>
          Dismiss
        </button>
      </div>
    </div>
  );
};

export default AdvisorPanel;
