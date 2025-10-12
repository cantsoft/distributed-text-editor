import React, { useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

export default function TextEdit(): React.JSX.Element {
  const canvas_ref  = useRef<HTMLCanvasElement | null>(null);
  const edit_ref    = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (canvas_ref.current === null || edit_ref.current === null) { return; }
    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);

    edit_ref.current.addEventListener("input", () => {
      sandbox.setUniform("u_cursor_pos", document.getSelection()?.getRangeAt(0).startOffset);
    });
  }, []);

  return (
    <>
      <canvas ref={canvas_ref} className="glslCanvas"/>
      <div ref={edit_ref} className="text-field" contentEditable="plaintext-only"/>;
    </>
  );
}