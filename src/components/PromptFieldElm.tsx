import { useState } from "react";
import Paper from "@mui/material/Paper";
import InputBase from "@mui/material/InputBase";
import { TextField } from "@mui/material";
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
      sx={{
        p: "0px 6px 0px 0px",
        m: "0 auto",
        display: "flex",
        alignItems: "center",
        width: 670,
        borderRadius: "0.75rem",
      }}
    >
      <TextField
        multiline
        sx={{
          px: 1,
          flex: 1,
          "& .MuiOutlinedInput-notchedOutline": {
            border: "none",
          },
        }}
        placeholder={standby ? "Send a message." : "Wait for Rinna..."}
        value={prompt}
        onChange={(e) => {
          setPrompt(e.target.value);
        }}
        onKeyPress={(e: React.KeyboardEvent<HTMLDivElement>) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            sendPrompt();
          }
        }}
      />
      {button}
    </Paper>
  );
}
