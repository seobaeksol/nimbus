import { Provider } from "react-redux";
import { store } from "./store";
import LayoutToolbar from "./components/Toolbar/LayoutToolbar";
import MultiPanelLayout from "./components/layout/MultiPanelLayout";
import ProgressContainer from "./components/common/ProgressContainer";
import "./App.css";
import { CommandProvider } from "./providers/CommandProvider";

function App() {
  return (
    <Provider store={store}>
      <CommandProvider>
        <div className="app">
          <LayoutToolbar />
          <MultiPanelLayout />
          <ProgressContainer />
        </div>
      </CommandProvider>
    </Provider>
  );
}

export default App;
