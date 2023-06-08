import { useState, useEffect } from "react";
import PromptFieldElm from "@/components/PromptFieldElm";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useMemo } from "react";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import ConversationElm from "@/components/ConversationElm";
import useRinnaStandby from "@/hooks/rinna-standby-hook";
import Grid from "@mui/material/Grid/Grid";

interface Query {
  id: number;
  prompt: string;
}

function App() {
  // https://amateur-engineer.com/react-mui-dark-mode/
  const prefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)");
  const theme = useMemo(
    () =>
      createTheme({
        palette: {
          mode: prefersDarkMode ? "dark" : "light",
        },
      }),
    [prefersDarkMode]
  );

  const [conversationList, setConversationList] = useState<Query[]>([]);
  const [rinnaStandby, setRinnaStandby] = useRinnaStandby();
  const [queryId, setQueryId] = useState(0);

  const send = (prompt: string) => {
    setConversationList([...conversationList, { id: queryId, prompt }]);
    setQueryId(queryId + 1);
  };

  return (
    <ThemeProvider theme={theme}>
      <Grid
        container
        sx={{
          width: "100%",
          alignItems: "center",
          justifyContent: "center",
          m: "0 auto",
        }}
      >
        {conversationList.map((conversation) => (
          <Grid
            item
            xs={12}
            sx={{
              alignItems: "center",
              justifyContent: "center",
            }}
            key={conversation.id}
          >
            <ConversationElm
              id={conversation.id}
              prompt={conversation.prompt}
            />
          </Grid>
        ))}
      </Grid>
      <Grid
        container
        sx={{
          position: "absolute",
          bottom: "0",
          width: "100%",
          padding: "20px",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <Grid
          item
          xs={12}
          sx={{ alignItems: "center", justifyContent: "center" }}
        >
          <PromptFieldElm
            send={send}
            standby={rinnaStandby}
            setStandby={setRinnaStandby}
          />
        </Grid>
      </Grid>
    </ThemeProvider>
  );
}

export default App;
