import React, { useEffect, useState } from "react";

import Taskbar from "./components/Taskbar";
import TextEdit from "./components/TextEdit";
import FileDialog from "./components/FileDialog";
import LoadingScreen from "./components/LoadingScreen";

import "./styles/Taskbar.css";
import "./styles/TextEdit.css";

export default function App(): React.JSX.Element {
  const [loaded, setLoaded] = useState<boolean>(false);
  const [dialog_active, setDialogActive] = useState<boolean>(false);

  useEffect(() => {
    setTimeout(() => {
      setLoaded(true)
    }, 2000);
  }, []);

  return (
    <>
      <Taskbar onSave={ loaded ? () => setDialogActive(true) : () => { alert("Can't save while loading"); }} />
      {
        loaded ? (
          <>
            <FileDialog
              active={dialog_active}
              onExit={() => setDialogActive(false)}
            />
            <TextEdit/>
          </>
        ) : (
          <>
            <LoadingScreen />
          </>
        )
      }
    </>
  );
}