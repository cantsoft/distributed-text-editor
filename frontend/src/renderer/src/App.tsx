import React, { useState } from "react";

import Taskbar from "./components/Taskbar";
import TextEdit from "./components/TextEdit";
import FileDialog from "./components/Filedialog";

import "./styles/Taskbar.css";
import "./styles/TextEdit.css";

export default function App(): React.JSX.Element {
  const [dialog_active, setDialogActive] = useState<boolean>(false);

  return (
    <>
      <FileDialog active={ dialog_active } onExit={ () => setDialogActive(false) }/>
      <Taskbar onSave={ () => setDialogActive(true) }/>
      <TextEdit/>
    </>
  );
}