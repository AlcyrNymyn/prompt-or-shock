# Configuration
All currently available configuration options are visible in the app. This includes some settings that must be set before pressing the start button:
 - VRChat Username. This is case insensitive.
 - PiShock Username. This is case insensitive.
 - PiShock API Key. You can generate the API key after logging in to your PiShock account by clicking Menu in the top right to open a menu dropdown, then Account to get to the menu page. Here, there will be a button to Generate API Key.
 - Shocker Share Code. This must be a share code either already claimed by you, or a code that is unclaimed. If it is unclaimed, it will be claimed by you once the application uses the code for a shock.

Once these values are entered, you can press the Start button. Changing these settings requires stopping and starting the program.

The remaining settings control how the program will shock you, and will be applied as you update them (no need to Stop/Start):
- Enable Warning Vibrate. When enabled, there will be a virbrate shortly before the shock.
- Life Loss Shock. This is the shock value used when you lose a life, i.e. when someone else's prompt is chosen (and you're still alive).
- Death Shock. This is the shock value used when the death animation is triggered at the end of the game, if you did not win.
- Death Shock Delay (seconds). This is the delay between when the program detects your death and when the shock is done. Since there is delay between your in-game death and when the shocker gets the command, this lets you set a delay so that the shock happens at a more opportune time. The value is an integer number of seconds.

All settings are automatically saved to a settings.json in the current working directory.