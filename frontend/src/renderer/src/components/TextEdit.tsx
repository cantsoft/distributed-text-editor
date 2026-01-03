import React, { useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

export default function TextEdit(): React.JSX.Element { const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const edit_ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (canvas_ref.current === null || edit_ref.current === null) {
      return;
    }

    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);

    const handleInput = (event: Event) => {
      const cursor_pos = document.getSelection()?.getRangeAt(0).startOffset;
      window.api.onUserInput(event.data, cursor_pos, event.inputType);
    };

    edit_ref.current.addEventListener("input", handleInput);

    return () => {
      edit_ref.current?.removeEventListener("input", handleInput);
      sandbox.destroy?.();
    };

  }, []);

  return (
    <>
      <canvas ref={canvas_ref} className="glslCanvas"/>
      <div ref={edit_ref} className="text-field" contentEditable="plaintext-only"/>
    </>
  );
}
