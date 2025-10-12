import React from "react";

import Taskbar from "./components/Taskbar";
import TextEdit from "./components/TextEdit";
import "./styles/Taskbar.css";
import "./styles/TextEdit.css"

export default function App(): React.JSX.Element {
  return (
    <>
      <Taskbar/>
      <TextEdit/>
    </>
  );
}