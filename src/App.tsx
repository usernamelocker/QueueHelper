import { useState, useEffect } from "react";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { AppProvider } from "./hooks/AppContext";
import { TranslationProvider } from "./i18n";
import NavSidebar from "./components/NavSidebar";
import TutorialOverlay from "./components/TutorialOverlay";
import Home from "./pages/Home";
import Settings from "./pages/Settings";
import Profiles from "./pages/Profiles";
import DraftRules from "./pages/DraftRules";
import Monitor from "./pages/Monitor";
import "./App.css";

const TUTORIAL_SEEN_KEY = "queue-helper-tutorial-seen";

function App() {
  const [showTutorial, setShowTutorial] = useState(false);
  const [firstTime, setFirstTime] = useState(false);

  useEffect(() => {
    const seen = localStorage.getItem(TUTORIAL_SEEN_KEY);
    if (!seen) {
      setFirstTime(true);
      setShowTutorial(true);
      localStorage.setItem(TUTORIAL_SEEN_KEY, "1");
    }
  }, []);

  return (
    <Router>
      <TranslationProvider>
      <AppProvider>
        <div className="flex h-screen">
          <NavSidebar
            onHelp={() => setShowTutorial(true)}
            showGlow={firstTime && !showTutorial}
          />
          <main className="flex-1 p-6 overflow-auto">
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="/profiles" element={<Profiles />} />
              <Route path="/draft-rules" element={<DraftRules />} />
              <Route path="/monitor" element={<Monitor />} />
            </Routes>
          </main>
        </div>
        {showTutorial && <TutorialOverlay onClose={() => { setShowTutorial(false); setFirstTime(false); }} />}
      </AppProvider>
      </TranslationProvider>
    </Router>
  );
}

export default App;
