# How the rat learns

This document describes the model in `crates/rattus-core`. Like *Sniffy*, Rattus is a teaching tool, not a research tool: the rat is a metaphor that makes the textbook phenomena visible. Where realism got in the way, it was sacrificed. The rat never becomes satiated, and it adapts to a new schedule of reinforcement much faster than a real rat. Every number below is a constant in the source and can be tuned; `tests/phenomena.rs` checks that the classic results still come out.

## Time

The simulation advances in fixed steps of 50 ms of **program time**. While the rat is visible, program time runs at the chosen animation speed (0.5× to 8× clock time). When the rat is **isolated**, it is not drawn and program time runs 60× to 3600× faster (600× by default), so a 30-trial classical conditioning experiment takes a few seconds. **Time off** in the home cage does not advance the program clock; it is marked on the record and applied to the rat's mind.

## Behaviour

The chamber is seen from the side. The rat moves between the water spout (left wall) and the lever and food cup (right wall). It performs one behaviour at a time, in bouts of a few seconds:

walking, sniffing, looking around, rearing, rearing against a wall, grooming, wiping its face, scratching, drinking, eating, pressing the bar, begging, rolling over, freezing, the startle response to shock and the orienting response to a new stimulus.

When a bout ends, the next behaviour is chosen in this order:

1. If the rat has just heard the dispenser click, it goes to the food cup. The probability is the strength of the **sound–food** association, so a rat that has been magazine trained runs to the cup at once.
2. If there is a pellet in the cup and the rat is there, it eats it.
3. It freezes with probability 0.9 × **fear**.
4. It presses the bar, or walks to the bar to press it, with a probability given by the bar-press tendency (below).
5. Otherwise it picks a behaviour at random. Each has a base weight (walking 22, sniffing 16, looking around 7, rearing 6, grooming 6, face wiping 3, scratching 3, begging 0.4, rolling over 0.25, rearing against a wall 8 when at a wall, drinking 6 at the spout, a stray bar press 0.5 at the lever), plus 40 × the behaviour's **action strength**. Walking goes preferably to places where the rat has been reinforced.

## Associations with inhibition

Every association has an excitatory and an inhibitory part; its strength is their difference. Learning that strengthens an association first removes inhibition and then adds excitation; extinction adds inhibition. During time off, inhibition halves every 24 hours. That is why an extinguished response comes back after a rest (**spontaneous recovery**), but extinguishes faster the second time.

## Operant conditioning

The three associations in the **Operant Associations** window:

- **Sound–food**: the click of the dispenser predicts food. Each time the rat eats a pellet, the click that announced it gains strength by 0.25 × e<sup>−delay/8 s</sup> × (1 − strength). The faster the rat gets to the cup, the more it learns, so magazine training accelerates as it goes. A click after which the rat finds the cup empty weakens it (rate 0.04). Clicks the rat ignores teach nothing.
- **Bar–sound**: pressing the bar produces the click. It gains 0.15 × (1 − strength) when a press is followed by the click.
- **Action strength**: how strongly each behaviour has been reinforced. When the dispenser operates, the behaviour in progress moves toward the current value of the click (the sound–food strength) at rate 0.2; behaviours that ended less than 2 s before get half that. Reinforcement also **generalizes** to similar behaviours, up to a fraction of the click's value: rearing ↔ rearing against a wall (0.5), rearing against a wall ↔ bar press (0.35), rearing ↔ begging (0.3), rearing ↔ bar press (0.15), grooming ↔ face wiping (0.4), grooming ↔ scratching (0.3), face wiping ↔ begging and scratching ↔ rolling over (0.25), grooming ↔ rolling over (0.2). This is what makes **shaping** work. Shaped behaviours that go unreinforced slowly lose strength (rate 0.03).

Because the reinforcer that acts immediately is the click, and the click's value is the sound–food strength, a rat that has not been magazine trained learns little from a pellet. With the click alone (**sound only**), pressing keeps being reinforced while sound–food slowly extinguishes, so it lasts much longer than in complete extinction (**secondary reinforcement**).

### Extinction and the partial reinforcement effect

An unreinforced press weakens bar–sound and the bar-press strength at rate 0.1 × *surprise*. Surprise is how much more reinforcement the rat expected than it is getting now:

- the difference between the long-term rate of reinforcement per press (it moves 0.01 per press) and the recent one (0.15 per press);
- plus 0.01;
- plus up to 0.05 when no reinforcement has come for much longer than usual.

After continuous reinforcement, the first unreinforced presses are very surprising and extinction is fast. After a lean variable-ratio schedule they are not, and the rat keeps pressing much longer (**partial reinforcement extinction effect**). Unexpected non-reinforcement also briefly raises frustration, which produces the extinction burst.

### Bar pressing and schedules

The bar-press **drive** is action strength × (0.4 + 0.6 × bar–sound). It becomes a tendency through a sigmoid, *d*² / (*d*² + 0.08), so a weak habit gives occasional presses and a strong one steady pressing. The tendency is then multiplied by (1 − fear)², which is the conditioned suppression measured in the CER.

The rat also learns about the schedule: averages of responses per reinforcement and of seconds between reinforcements, with their variability, and whether reinforcement depends on responses (ratio) or on time (interval). Each reinforcement weighs 0.25.

- When the number of responses per reinforcement is predictable, as on **fixed ratio** schedules, pressing pauses after each reinforcement for 0.3 s per response required. This is the **post-reinforcement pause**, followed by a fast run.
- When the interval is predictable, as on **fixed interval** schedules, the tendency grows as (elapsed / interval)². This gives the **scallop**.
- On ratio schedules, runs of presses are longer than on interval schedules, so **VR** gives higher rates than **VI**.
- Shortly after reinforcement on intermittent schedules the rat drinks more (**adjunctive behaviour**).

## Classical conditioning (CER)

A classical conditioning experiment has **stages**; each stage has one or more trial types (the conditioned stimuli — light, tone at 60–100 dB, bell — and the shock: none, low, medium or high), a number of trials, and a mean inter-trial interval (each interval varies by ±50%, never less than a minute). Every trial presents the CS for 30 s; the shock occupies the last second. The rat keeps pressing the bar on its operant schedule throughout, which is why the classic CER experiment uses a rat trained to press on VR-25 (File ▸ New Bar-Trained Rat).

After each trial the strengths of the stimuli present change by the **Rescorla–Wagner** rule:

ΔV = α · β · (λ − ΣV)

- α is the salience of the stimulus: light 0.30, bell 0.35, tone (dB − 50) / 100, between 0.05 and 0.5 (louder tones condition faster);
- λ is the strength the shock can support: low 0.55, medium 0.85, high 1.0 (none 0);
- β is 0.5 with shock and 0.25 without;
- ΣV is the summed strength of all stimuli present, so compound stimuli show blocking and overshadowing.

**CS Response Strength** shows each V. During the CS, **fear** rises (time constant 0.8 s) toward ΣV and falls back slowly afterwards (8 s); a shock itself also frightens. Fear makes the rat freeze and suppresses bar pressing.

**Pain sensitivity** predicts how long the next unconditioned response to shock lasts (0.8 s + 1.2 s × sensitivity × shock in mA). Strong shocks sensitize it, weak ones habituate it, and it returns toward 1 during time off. The **orienting response** to a stimulus habituates: its probability is multiplied by 0.7 at each presentation and recovers during time off.

For every trial the app records:

- **suppression ratio** = presses during the CS / (presses during the CS + presses in the 30 s before it);
- **movement ratio** = the same ratio computed with the time spent moving instead of presses.

0.5 means the CS changed nothing; 0 means complete suppression (complete freezing).

## References

- Annau, Z., & Kamin, L. J. (1961). The conditioned emotional response as a function of intensity of the US. *Journal of Comparative and Physiological Psychology, 54*, 428–432.
- Estes, W. K., & Skinner, B. F. (1941). Some quantitative properties of anxiety. *Journal of Experimental Psychology, 29*, 390–400.
- Falk, J. L. (1961). Production of polydipsia in normal rats by an intermittent food schedule. *Science, 133*, 195–196.
- Ferster, C. B., & Skinner, B. F. (1957). *Schedules of Reinforcement*. Appleton-Century-Crofts.
- Kamin, L. J. (1968). Attention-like processes in classical conditioning. In M. R. Jones (Ed.), *Miami Symposium on the Prediction of Behavior: Aversive Stimulation*. University of Miami Press.
- Rescorla, R. A., & Wagner, A. R. (1972). A theory of Pavlovian conditioning: Variations in the effectiveness of reinforcement and nonreinforcement. In A. H. Black & W. F. Prokasy (Eds.), *Classical Conditioning II* (pp. 64–99). Appleton-Century-Crofts.
- Skinner, B. F. (1938). *The Behavior of Organisms*. Appleton-Century-Crofts.
