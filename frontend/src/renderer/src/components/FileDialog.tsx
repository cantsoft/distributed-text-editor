import { useEffect, useRef } from "react";

import "../styles/FileDialog.css";

export default function FileDialog({
    active,
    onExit
}: {
    active: boolean,
    onExit: () => void
}): React.JSX.Element {
    let container_ref = useRef<HTMLDivElement | null>(null);
    let modal_ref = useRef<HTMLDivElement | null>(null);
    let email_input_ref = useRef<HTMLDivElement | null>(null);

    const handleClick = (event: MouseEvent) => {
        if (modal_ref.current && !modal_ref.current.contains(event.target)) {
            onExit();
        }
    };

    const onSave = () => {
        if (!email_input_ref || !email_input_ref.current) { return; }

        const content = email_input_ref.current.textContent.replace(/[\n\r\t]/gm, "") 
            || email_input_ref.current.innerText.replace(/[\n\r\t]/gm, "") 
            || ""

        if (content == "") { 
            alert("Filename cannot be empty");
            return;
        }
        
        window.api.save(content + ".dte");
        alert(`Saved as ${content}.dte`);
    }

    useEffect(() => {
        window.addEventListener("mousedown", handleClick);
        return () => { window.removeEventListener("mousedown", handleClick); }
    }, []);
    
    return (
        <div className={ active ? "modal-container active" : "modal-container" } ref={ container_ref }>
            <div className="order-modal" ref={ modal_ref }>
                <div className="textfield">
                    <h3>Filename:</h3>
                    <div contentEditable="true" className="text-input" ref={ email_input_ref }></div>                    
                </div>
                <div className="save-btn" onClick={ () => {
                    onExit();
                    onSave();
                    } }>Save</div>
            </div>
        </div>
    )
}