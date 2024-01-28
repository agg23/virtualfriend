import getPixels from "get-pixels";
import { argv } from "process";
import { writeFileSync } from "fs";

if (argv.length != 5) {
  console.log(`Received ${argv.length - 2} arguments. Expected 3\n`);
  console.log(
    "Usage: node convertToVf.js [input left path] [input right path] [output path]"
  );

  process.exit(1);
}

const inputLeftPath = argv[2];
const inputRightPath = argv[3];
const outputPath = argv[4];

let outputBuffer = Buffer.alloc(384 * 224 * 2);

const pixels = async (path, offset) =>
  new Promise((resolve) => {
    getPixels(path, (err, pixels) => {
      if (err) {
        console.log(`Error loading pixels: ${err}`);
      }

      const [width, height] = pixels.shape;

      for (let x = 0; x < width; x++) {
        for (let y = 0; y < height; y++) {
          const red = pixels.get(x, y, 0);

          outputBuffer[y * width + x + offset] = red;
        }
      }

      resolve();
    });
  });

const main = async () => {
  await pixels(inputLeftPath, 0);
  await pixels(inputRightPath, 384 * 224);

  writeFileSync(outputPath, outputBuffer, { flag: "w" });
  console.log(`Wrote output file ${outputPath}`);
};

main();
