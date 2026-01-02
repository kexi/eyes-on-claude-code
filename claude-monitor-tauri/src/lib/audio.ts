let audioContext: AudioContext | null = null;

const getAudioContext = (): AudioContext => {
  if (!audioContext) {
    audioContext = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
  }
  return audioContext;
};

const playTone = (
  frequency: number,
  duration: number,
  type: OscillatorType = 'sine',
  volume = 0.3
): void => {
  try {
    const ctx = getAudioContext();
    const oscillator = ctx.createOscillator();
    const gainNode = ctx.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(ctx.destination);

    oscillator.type = type;
    oscillator.frequency.value = frequency;
    gainNode.gain.value = volume;

    oscillator.start(ctx.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);
    oscillator.stop(ctx.currentTime + duration);
  } catch (e) {
    console.error('Failed to play sound:', e);
  }
};

// Completion sound: Pleasant ascending two-tone chime
export const playCompletionSound = (): void => {
  playTone(523.25, 0.15, 'sine', 0.2); // C5
  setTimeout(() => {
    playTone(783.99, 0.25, 'sine', 0.2); // G5
  }, 120);
};

// Waiting/Attention sound: Soft double beep
export const playWaitingSound = (): void => {
  playTone(440, 0.1, 'sine', 0.15); // A4
  setTimeout(() => {
    playTone(440, 0.1, 'sine', 0.15); // A4
  }, 150);
};
