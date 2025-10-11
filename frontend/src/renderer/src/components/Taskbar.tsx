import React from "react";

export default function Taskbar(): React.JSX.Element{
  return (
    <div className="taskbar-container">
      <div className="window-menu">
        <button className="menu-button" onClick={() => alert("File clicked")}>File</button>
        <button className="menu-button" onClick={() => alert("Edit clicked")}>Edit</button>
      </div>
      <div className="window-title">Distributed Text Editor</div>
      <div className="window-controls">
        <button className="controls-button minimise" onClick={() => alert("Minimise clicked")}>-</button>
        <button className="controls-button maximise" onClick={() => alert("Maximise clicked")}>a</button>
        <button className="controls-button exit" onClick={() => alert("Exit clicked")}>x</button>
      </div>
    </div>
  );
}