import { queryRinna } from "@/hooks/rinna-hook";
import { Grid } from "@mui/material";
import AssistantIcon from "@mui/icons-material/Assistant";
import PersonIcon from "@mui/icons-material/Person";
import Avatar from "@mui/material/Avatar";
import { blue, green } from "@mui/material/colors";
import Box from "@mui/material/Box";

interface ConversationTentativeElmProps {
  id: number;
  prompt: string;
}

export default function ConversationTentativeElm({
  id,
  prompt,
}: ConversationTentativeElmProps) {
  const rinnaResponse = queryRinna(id, prompt)
    .replace(/\n/g, "")
    .replace(/\r/g, "")
    .replace(/<NL>/g, "\n");

  // replace </s>
  const text = rinnaResponse.includes("</s>")
    ? rinnaResponse.replace(/<\/s>/g, "")
    : `${rinnaResponse}▊`;

  // convert \n to <br />
  const content = text.split("\n").map((str, index) => (
    <span key={index}>
      {str}
      <br />
    </span>
  ));

  return (
    <>
      <Grid
        container
        sx={{
          width: "768px",
          justifyContent: "flex-start",
          m: "0 auto",
        }}
        spacing={2}
      >
        <Grid
          item
          xs={1}
          sx={{
            display: "flex",
            flexDirection: "row-reverse",
          }}
        >
          <Avatar sx={{ bgcolor: blue[500] }} variant="rounded">
            <PersonIcon />
          </Avatar>
        </Grid>
        <Grid item xs={11}>
          <Box
            sx={{
              p: 1,
            }}
          >
            {prompt}
          </Box>
        </Grid>
      </Grid>

      <Grid
        container
        sx={{
          width: "768px",
          justifyContent: "flex-start",
          m: "0 auto",
        }}
        spacing={2}
      >
        <Grid
          item
          xs={1}
          sx={{
            display: "flex",
            flexDirection: "row-reverse",
          }}
        >
          <Avatar sx={{ bgcolor: green[500] }} variant="rounded">
            <AssistantIcon />
          </Avatar>
        </Grid>
        <Grid item xs={11}>
          <Box
            sx={{
              p: 1,
            }}
          >
            {content}
          </Box>
        </Grid>
      </Grid>
    </>
  );
}
