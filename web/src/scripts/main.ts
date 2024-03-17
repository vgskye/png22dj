import { Context, Image, type ProcessedImage } from "png22dj";

const upload = document.getElementById("upload") as HTMLInputElement;
const name = document.getElementById("name") as HTMLInputElement;
const width = document.getElementById("width") as HTMLInputElement;
const height = document.getElementById("height") as HTMLInputElement;
const preview = document.getElementById("preview") as HTMLButtonElement;
const save = document.getElementById("save") as HTMLButtonElement;
const canvas = document.getElementById("canvas") as HTMLCanvasElement;

const ctx = new Context();

let cachedImageFile: File | undefined;
let cachedImage: Image | undefined;
let processed: ProcessedImage | undefined;
let processedWidth = -1;
let processedHeight = -1;

preview.addEventListener("click", processImage);

async function processImage() {
  if (upload.files!!.length == 0) {
    alert("please upload a source image");
    return;
  }
  const w = width.valueAsNumber;
  const h = height.valueAsNumber;
  const file = upload.files!![0];

  if (file != cachedImageFile || !cachedImage) {
    if (cachedImage) {
      console.log("freeing previous image as file changed");
      cachedImage.free();
    }
    console.log("creating image");
    const buf = new Uint8Array(await file.arrayBuffer());
    cachedImage = new Image(buf);
  }
  console.log("processing image");
  if (processed) {
    processed.free();
  }
  processed = ctx.process(cachedImage, w, h);
  processedWidth = w;
  processedHeight = h;

  console.log("drawing preview");
  const x = Math.floor((128 * w - processed.width) / 2);
  const y = Math.floor((128 * h - processed.height) / 2);

  canvas.width = w * 128;
  canvas.height = h * 128;

  const canvasCtx = canvas.getContext("2d")!!;

  const data = canvasCtx.createImageData(processed.width, processed.height);
  const clamped = data.data;
  const uint = new Uint8Array(
    clamped.buffer,
    clamped.byteOffset,
    clamped.byteLength,
  );
  processed.draw(uint);

  canvasCtx.putImageData(data, x, y);
  console.log("drew preview");
}

save.addEventListener("click", async () => {
  if (upload.files!!.length == 0) {
    alert("please upload a source image");
    return;
  }
  const w = width.valueAsNumber;
  const h = height.valueAsNumber;
  const file = upload.files!![0];

  if (
    file != cachedImageFile ||
    !processed ||
    w != processedWidth ||
    h != processedHeight
  ) {
    await processImage();
  }

  const title = name.value;
  console.log("serializing");
  const result = processed!!.serialize(title != "" ? title : undefined);

  console.log("creating URL");
  const blob = new Blob([result], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  console.log("created URL", url);

  console.log("starting download the jank way");
  const elem = document.createElement("a");
  elem.href = url;
  elem.download = file.name.split(".").slice(0, -1).join(".") + ".2dja";
  document.body.appendChild(elem).click();
  elem.remove();
});
