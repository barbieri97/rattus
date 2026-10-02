# Exercises

These exercises cover the basic phenomena of operant and classical conditioning. Each one says what to do and what to look for. Save your rat (File ▸ Save) at the end of each exercise: later exercises start from a trained rat.

Useful controls:

- **Space** or a click on the lever gives a pellet by hand.
- **Ctrl+I** (⌘I on macOS) isolates the rat to accelerate time; press it again to watch.
- **Ctrl+M** marks the cumulative record.
- **P** pauses.
- **Edit ▸ Copy** (Ctrl+C) copies the data of the selected window, ready to paste into a spreadsheet.

## 1. Magazine training

A new rat does not know that the click of the dispenser means food.

1. File ▸ New Naive Rat. Open the Operant Associations window.
2. Give a pellet (Space) while the rat is away from the food cup. Wait until it has found and eaten the pellet before giving the next one.
3. Continue until the rat turns and runs to the cup as soon as it hears the click, usually after 30 to 50 pellets.

**What to look for:** Sound–Food climbs slowly at first and then quickly, because a rat that comes to the cup sooner learns more from each pellet. The click has become a conditioned (secondary) reinforcer.

## 2. Shaping bar pressing

1. Start from your magazine-trained rat. By default, bar presses are reinforced continuously (Experiment ▸ Design Operant Conditioning Experiment shows CRF, food).
2. Give a pellet whenever the rat is on the right half of the chamber facing the lever.
3. Once it stays there, reinforce only rearing near the lever, then only touching the lever.
4. As soon as the rat presses on its own the dispenser does the work; stop giving pellets.

**What to look for:** Bar–Sound and Action Strength rise and the cumulative record becomes steep and steady. Compare with a magazine-trained rat left alone on CRF: it eventually discovers the lever, but usually much later.

## 3. Cumulative records

The cumulative record pen steps up at each press and moves right with time, so the slope is the response rate. Ticks mark reinforcements; the pen returns to the bottom when the paper is full. Use the zoom buttons, drag the record to scroll back, and mark interesting moments with Ctrl+M.

## 4. Extinction

1. With a rat that presses steadily on CRF, choose Reinforcer: **None** in the operant design dialog.
2. Isolate the rat and let an hour or two of program time pass.

**What to look for:** a short burst of pressing, then the record flattens. Bar–Sound and Action Strength fall.

## 5. Spontaneous recovery

1. After extinction, choose Experiment ▸ Remove Rat for Time Off (24 hours).
2. Watch the first minutes after the rat returns.

**What to look for:** pressing comes back for a while, then extinguishes again, faster than the first time.

## 6. Secondary reinforcement

1. Train a rat on CRF and save it. Extinguish it with Reinforcer: **None** for one hour and note the number of presses (Behavior Log, or Edit ▸ Copy on the cumulative record).
2. Reopen the saved rat and extinguish it with Reinforcer: **Sound only**.

**What to look for:** the click alone keeps the rat pressing much longer: it is still a reinforcer, until Sound–Food extinguishes too.

## 7. Schedules of reinforcement

Move a trained rat gradually to leaner schedules (for example VR-5, then VR-10, then VR-25), giving it some time on each, and compare:

- **Fixed ratio (FR):** a pause after each reinforcement, then a fast run of presses.
- **Variable ratio (VR):** a high, steady rate.
- **Fixed interval (FI):** little pressing after reinforcement, accelerating as the interval ends (the scallop).
- **Variable interval (VI):** a moderate, steady rate.

Then extinguish one rat trained on CRF and one trained on VR-25: the second keeps pressing much longer (**partial reinforcement extinction effect**).

## 8. Shaping other behaviours

The rat can learn to beg, wipe its face or roll over. Tick "Show the strength of every action" in Operant Associations and watch the Behavior Log.

- **Begging:** reinforce rearing until the rat rears a lot, then reinforce only begging.
- **Face wiping:** reinforce grooming, then face wiping.
- **Rolling over:** reinforce grooming and scratching, then rolling over.

## 9. Classical conditioning: acquisition of the CER

1. File ▸ New Bar-Trained Rat: this rat presses steadily on a VR-25 schedule.
2. Experiment ▸ Design Classical Conditioning Experiment: one stage with 10 trials of Light and Medium shock, mean interval 5 minutes. Click **Save and Run**, then isolate the rat.

**What to look for:** in Suppression Ratio, the first trial is near 0.5 (the light alone changes nothing) and the ratio drops toward 0 as CS Response Strength for the light grows. During the light the rat freezes: Fear rises in Sensitivity & Fear, and the cumulative record goes flat while the CS strip shows the light.

## 10. Extinction and spontaneous recovery of the CER

1. Add a second stage with 30 trials of Light and **No shock**.
2. After the experiment, give the rat 24 hours of time off and run a few more Light-alone trials.

**What to look for:** the suppression ratio climbs back toward 0.5 during extinction, then drops again after the rest.

## 11. Stimulus intensity

Repeat the acquisition with a tone at 70 dB and at 95 dB, or with low and high shock.

**What to look for:** louder tones condition faster; stronger shocks condition faster and to a higher level.
