import React, { RefObject, useEffect, useRef, useState } from "react";

type DropdownOption = {label: string, action: () => void}

const menus = [
  {
    label: "File",
    options: [
      { label: "New...",      action: () => alert("New...")     },
      { label: "Open",        action: () => alert("Open")       },
      { label: "Save",        action: () => alert("Save")       },
      { label: "Save As...",  action: () => alert("Save As...") },
      { label: "Exit",        action: window.api.close          }          
    ]
  },
  {
    label: "Edit",
    options: [
      { label: "Undo",  action: () => alert("Undo") },
      { label: "Redo",  action: () => alert("Redo") }
    ]
  },
];


function DropdownMenu({
  ref,
  options,
  parent_id
}: {
  ref: RefObject<HTMLDivElement | null> | undefined,
  options: DropdownOption[],
  parent_id: string
}): React.JSX.Element {
  useEffect(() => {
    let parent: HTMLElement | null = document.getElementById(parent_id);
    if (ref === undefined || ref.current === null || parent === null) { return; }

    let parentbbox: DOMRect = parent.getBoundingClientRect();
    ref.current.style.left = `${parentbbox.left}px`;
    ref.current.style.top = `${parentbbox.bottom}px`;
  }, [ref]);

  return(
    <div className={ref !== undefined ? "dropdown-menu active" : "dropdown-menu"} ref={ref}>
      {
        options.map((option: DropdownOption, option_index: number) => {
          return (
            <div 
              className="menu-option"
              onClick={option.action} 
              key={option_index}
            >{option.label}</div>
          );
        })
      }
    </div>
  );
}


export default function Taskbar(): React.JSX.Element{
  const [activeMenu, setActiveMenu] = useState<number | null>(null);
  const activeMenuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const onClick = (event: MouseEvent) => {
      if (activeMenu !== null
        && activeMenuRef.current 
        && !activeMenuRef.current.contains(event.target as Node)
      ) { setActiveMenu(null); }
    };

    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [activeMenu]);

  return (
    <div className="taskbar-container">
      <div className="window-menu">
        {
          menus.map((menu_data, menu_index) =>{
            let button_id = `${menu_data.label}-menu-button`;
            return(
              <div className="menu-wrapper" key={menu_index}>
                <button 
                  id={button_id}
                  className="menu-button"
                  onClick={() => setActiveMenu(menu_index)}
                  onMouseEnter={() => {
                    if (activeMenu !== null) { setActiveMenu(menu_index) }
                  }
                  }
                >{menu_data.label}</button>
                <DropdownMenu
                  ref={activeMenu === menu_index ? activeMenuRef : undefined}
                  options={menu_data.options}
                  parent_id={button_id}
                />
              </div>
            );
          })
        }
      </div>
      <div className="window-title">Distributed Text Editor</div>
      <div className="window-controls">
        <button className="controls-button minimise" onClick={window.api.minimize}>-</button>
        <button className="controls-button maximise" onClick={window.api.maximize}>□</button>
        <button className="controls-button exit" onClick={window.api.close}>x</button>
      </div>
    </div>
  );
}