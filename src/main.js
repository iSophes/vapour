import * as slint from "slint-ui";
let ui = slint.loadFile(new URL("../ui/main.slint", import.meta.url));
let app = new ui.AppWindow();

app.time = "6:00 AM";
app.balance = "5.21";
app.hello_text = "Hello Sophie!";
app.currentMenu = "startup";

await app.run();
