import "./App.css";
import ModManager from "./components/ModManager";
import { ThemeProvider } from "./components/theme/theme-provider";
import { DownloadProvider } from "./components/context/DownloadContext";
import GlobalErrorHandler from "./components/GlobalErrorHandler";

function App() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="bzmm-ui-theme">
      <DownloadProvider>
        <main className="h-screen bg-background text-foreground overflow-hidden">
          <ModManager />
          <GlobalErrorHandler />
        </main>
      </DownloadProvider>
    </ThemeProvider>
  );
}

export default App;
