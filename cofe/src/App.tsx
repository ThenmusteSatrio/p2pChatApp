import { useState } from "react";
import "./App.css";
import MainLayout from "./MainLayout";
import SplashScreen from "./SplashScreen";

function App() {
  const [ready, setReady] = useState(false);
  return ready ? (
    <MainLayout/>
  ): (
    <SplashScreen onDone={() => setReady(true)}/>
  );
}


export default App;
