import Header from "@/components/Header";
import MainContent from "@/components/MainContent";

function App() {
  return (
    <div className="min-h-screen w-screen flex flex-col">
      {/* Header with app title, folder selection, and cache status */}
      <Header
        folderPath={""}
        cacheActionText={{}}
        onPickFolder={() => Promise.resolve()}
      />

      {/* Main content area with media grid */}
      <MainContent
        folderPath={""}
        mediaItems={[]}
      />
    </div>
  );
}

export default App;
