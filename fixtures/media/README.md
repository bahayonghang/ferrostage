# Original media regression fixtures

The synthetic twelve-second clips, three-frame GIF/WebP, public/private animation and media PDFs were generated for FerroStage task11. Original creation/readback receipts and recipes remain in that task research; these stable product-test inputs are exact copies, verified by identities.json. No user file is modified by tests. GIF/WebP delays are120/240/360ms. Video is320×180 at30fps, with generated audio. Background tests always use fakesink, never speakers or desktop windows.

media-smoke.pdf has external GIF/WebP, MP4 start2/duration8/loop, public Ogg audio, embedded real MP4 manual activation and a missing file. media-crop-smoke.pdf requires explicit Notes Right: page1 straddles the split; page2 is notes-only autostart audio; page3 is public audio. movie.pdf is the existing real Beamer-compiled fixtures/beamer/movie.pdf, paired here with the generated playable fixture-video.mp4. The separate older media.pdf contains intentionally invalid embedded video and is not playback evidence.

The GIF/WebP fixture has independently known BGRA colors and exact delays; WebP alpha compositing may differ by one RGB level. The privacy GIF has public red/orange left pixels and private green/cyan right pixels.
