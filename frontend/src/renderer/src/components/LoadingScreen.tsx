import { useEffect, useRef } from "react";

import "../styles/LoadingScreen.css"

export default function LoadingScreen() {
    return (
        <div className="loading-screen-container">
            <div>
                Loading...
            </div>
        </div>
    );
}