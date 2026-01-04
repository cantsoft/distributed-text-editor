import React, { useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

export default function TextEdit(): React.JSX.Element {
  const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const edit_ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (canvas_ref.current === null || edit_ref.current === null) {
      return;
    }

    window.api.onRemoveRequest((position: number) => {
      const content = edit_ref.current!.textContent;
      edit_ref.current!.textContent = content.slice(0, position - 1) + content.slice(position);
    });

    window.api.onInsertRequest((position: number, char: string) => {
      const content = edit_ref.current!.textContent;
      edit_ref.current!.textContent = content.slice(0, position) + char + content.slice(position);
    });

    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);

    const handleKeyDown = (event: KeyboardEvent): void => {
      const cursor_pos = document.getSelection()?.getRangeAt(0).startOffset;
      window.api.onUserKeydown(event.key, cursor_pos);
    };

    edit_ref.current.addEventListener("keydown", handleKeyDown);

    return () => {
      edit_ref.current?.removeEventListener("keydown", handleKeyDown);
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