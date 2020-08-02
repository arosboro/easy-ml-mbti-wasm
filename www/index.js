import { train } from 'mnist-wasm'

let trainButton = document.querySelector('#train')
trainButton.disabled = true

let canvas = document.querySelector('#image');
canvas.width = 28
canvas.height = 28
let context = canvas.getContext('2d')

let currentImageIndicator = document.querySelector('#currentImage');
let image = 0;
let mnist = null;
let training = null;
let testing = null;

let nextButton = document.querySelector('#next')
nextButton.disabled = true;
nextButton.addEventListener('click', () => {
    image += 1;
    drawCurrentImage()
})
let previousButton = document.querySelector('#previous')
previousButton.disabled = true;
previousButton.addEventListener('click', () => {
    image -= 1;
    drawCurrentImage()
})

let drawCurrentImage = () => {
    image = Math.min(Math.max(0, image), 7999)
    if (mnist && context) {
        mnist.draw(training.images[image], context)
    }
    currentImageIndicator.innerHTML = `Image #${image}: (${training.labels[image]})`
}

// The mnist package doesn't fetch the data over the network, it's actually
// just in memory. This is great for simplifying this example, but causes
// problems with the template setup of auto recompiling the JavaScript each
// time a file is modified. Moving the import to happen dynamically after
// the user clicks a button prevents the browser doing too much work in the
// background while editing files with the webserver open.
let prepareButton = document.querySelector('#prepare')
prepareButton.addEventListener('click', async () => {
    mnist = await import('mnist')

    let dataset = mnist.set(8000, 2000)

    training = splitData(dataset.training)
    testing = splitData(dataset.test)

    drawCurrentImage()

    nextButton.disabled = false;
    previousButton.disabled = false;
    trainButton.disabled = false;
})

/**
 * Converts a dataset provided by the mnist package into two seperate
 * arrays, the first, an array of images, and the second an array of labels.
 */
let splitData = (dataset) => {
    let labels = []
    let images = []
    for (let entry of dataset) {
        images.push(entry.input)
        // dataset is encoded as 1-hot, ie an image of 5 is represented as
        // [0 0 0 0 0 1 0 0 0 0], convert this to a single digit as data
        // transfer over wasm is slow
        labels.push(entry.output.indexOf(1))
    }
    return {
        labels: labels,
        images: images,
    }
}

trainButton.addEventListener('click', () => {
  train()
})
