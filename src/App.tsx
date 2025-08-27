import { Provider } from "react-redux";
import { store } from "./store";
import LayoutToolbar from "./components/Toolbar/LayoutToolbar";
import MultiPanelLayout from "./components/layout/MultiPanelLayout";
import ProgressContainer from "./components/common/ProgressContainer";
import "./App.css";

function App() {
  return (
    <Provider store={store}>
      <div className="app">
        <LayoutToolbar />
        <MultiPanelLayout />
        <ProgressContainer />
      </div>
    </Provider>
  );
}

export default App;