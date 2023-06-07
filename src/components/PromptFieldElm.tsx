import { useState } from "react";
import Paper from "@mui/material/Paper";
import InputBase from "@mui/material/InputBase";
import Divider from "@mui/material/Divider";
import IconButton from "@mui/material/IconButton";
import SendIcon from "@mui/icons-material/Send";
import PendingIcon from "@mui/icons-material/Pending";

interface PromptFieldElmProps {
  send: (prompt: string) => void;
  standby: boolean;
  setStandby: (standby: boolean) => void;
}

export default function PromptFieldElm({
  send,
  standby,
  setStandby,
}: PromptFieldElmProps) {
  const [prompt, setPrompt] = useState("");

  const sendPrompt = () => {
    send(prompt);
    setStandby(false);
    setPrompt("");
  };

  const button = standby ? (
    <IconButton
      color="primary"
      sx={{ p: "10px" }}
      aria-label="send"
      onClick={(_e) => {
        sendPrompt();
      }}
    >
      <SendIcon />
    </IconButton>
  ) : (
    <IconButton color="primary" sx={{ p: "10px" }} aria-label="wait">
      <PendingIcon />
    </IconButton>
  );

  return (
    <Paper
      component="form"
      sx={{ p: "2px 4px", display: "flex", alignItems: "center", width: 400 }}
    >
      <InputBase
        sx={{ ml: 1, flex: 1 }}
        placeholder="Send a message."
        inputProps={{ "aria-label": "Send a message." }}
        value={prompt}
        onChange={(e) => {
          setPrompt(e.target.value);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            sendPrompt();
          }
        }}
      />
      <Divider sx={{ height: 28, m: 0.5 }} orientation="vertical" />
      {button}
    </Paper>
  );
}
