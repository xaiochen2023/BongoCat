interface AudioCache {
  [key: string]: AudioBuffer;
}

const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
const audioCache: AudioCache = {};
let soundsPreloaded = false;

// Define sound paths relative to the public directory
const soundPaths = {
  keypress: '/assets/sounds/keypress.mp3',
  mouseclick: '/assets/sounds/mouseclick.mp3',
};

async function loadSound(name: string, url: string): Promise<AudioBuffer | null> {
  if (audioCache[name]) {
    return audioCache[name];
  }
  try {
    const response = await fetch(url);
    const arrayBuffer = await response.arrayBuffer();
    const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
    audioCache[name] = audioBuffer;
    return audioBuffer;
  } catch (error) {
    console.error(`Failed to load sound: ${name} from ${url}`, error);
    return null;
  }
}

export async function preloadSounds(): Promise<void> {
  if (soundsPreloaded) return;

  const loadPromises: Promise<AudioBuffer | null>[] = [];
  for (const [name, path] of Object.entries(soundPaths)) {
    loadPromises.push(loadSound(name, path));
  }

  await Promise.all(loadPromises);
  soundsPreloaded = true;
  console.log('All sounds preloaded.', audioCache);
}

export function playSound(name: keyof typeof soundPaths): void {
  if (!audioContext || !soundsPreloaded) {
    console.warn('AudioContext not available or sounds not preloaded. Cannot play sound:', name);
    return;
  }

  const audioBuffer = audioCache[name];
  if (audioBuffer) {
    const source = audioContext.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(audioContext.destination);
    source.start(0);
  } else {
    console.warn(`Sound not found in cache: ${name}. Attempting to load dynamically...`);
    // Attempt dynamic load if not in cache (e.g. if preload failed or was skipped)
    loadSound(name, soundPaths[name]).then(buffer => {
      if (buffer) {
        const source = audioContext.createBufferSource();
        source.buffer = buffer;
        source.connect(audioContext.destination);
        source.start(0);
      }
    });
  }
}

// Initialize and preload sounds when the module is loaded
// This ensures sounds are ready as early as possible.
// Alternatively, call preloadSounds() explicitly at app startup.
if (audioContext) { // Ensure AudioContext is available before preloading
    preloadSounds();
}
