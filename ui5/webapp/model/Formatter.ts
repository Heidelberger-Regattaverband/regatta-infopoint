import Log from "sap/base/Log";
import ResourceBundle from "sap/base/i18n/ResourceBundle";
import { IndicationColor } from "sap/ui/core/library";
import {
  Race,
  Heat,
  Entry,
  HeatEntry,
  Athlete,
  Club,
  Crew,
  RaceState,
  HeatState,
  RoundCode,
  GroupValue
} from "./types";
import { Priority } from "sap/m/library";

/**
 * XML-binding formatters for regatta labels, backend state codes, and highlight colors.
 * The component injects a shared resource bundle; localized formatters return keys until it is available.
 *
 * @namespace de.regatta_hd.infoportal.model
 */
export default class Formatter {
  /**
   * The shared i18n {@link ResourceBundle}, injected by `Component#init` via
   * {@link Formatter.init}. Optional so the class can be instantiated/loaded
   * before the component is fully initialised (the `i18n` method falls back
   * to returning the raw key in that case).
   */
  private static bundle?: ResourceBundle;

  /**
   * Injects the shared i18n {@link ResourceBundle} (typically obtained from
   * `Component#getModel("i18n").getResourceBundle()`). Idempotent — subsequent
   * calls overwrite the reference, which lets tests provide a stub bundle.
   *
   * @param bundle - Already loaded bundle shared with the application component.
   */
  static init(bundle: ResourceBundle): void {
    Formatter.bundle = bundle;
  }

  /**
   * Maps backend priority values 1 through 3 to low, medium, and high; all other values map to none.
   *
   * @param priority - Optional numeric priority code from the backend.
   * @returns The matching UI5 priority, or `Priority.None` for missing or unrecognized codes.
   */
  static priority(priority?: number): Priority {
    switch (priority) {
      case 3:
        return Priority.High;
      case 2:
        return Priority.Medium;
      case 1:
        return Priority.Low;
      default:
        return Priority.None;
    }
  }

  /**
   * Formats a date-time string as local browser time in `DD.MM.YYYY HH:mm` format; missing input yields empty text.
   *
   * @param dateTime - Optional date-time string interpreted by the browser's `Date` constructor.
   * @returns Local date/time text, or an empty string for missing input; malformed dates are not validated.
   */
  static dateTime(dateTime?: string): string {
    if (!dateTime) {
      return "";
    }

    const date = new Date(dateTime);
    const day = date.getDate().toString().padStart(2, "0");
    const month = (date.getMonth() + 1).toString().padStart(2, "0");
    const year = date.getFullYear();
    const hours = date.getHours().toString().padStart(2, "0");
    const minutes = date.getMinutes().toString().padStart(2, "0");

    return `${day}.${month}.${year} ${hours}:${minutes}`;
  }

  /**
   * Formats a timestamp in local browser time, including seconds and milliseconds; missing input yields empty text.
   *
   * @param timestamp - Optional timestamp string interpreted by the browser's `Date` constructor.
   * @returns Local time in `DD.MM.YYYY HH:mm:ss.SSS` format, or empty text for missing input.
   * Invalid dates are not checked.
   */
  static timestamp(timestamp?: string): string {
    if (!timestamp) {
      return "";
    }

    const date = new Date(timestamp);
    const day = date.getDate().toString().padStart(2, "0");
    const month = (date.getMonth() + 1).toString().padStart(2, "0");
    const year = date.getFullYear();
    const hours = date.getHours().toString().padStart(2, "0");
    const minutes = date.getMinutes().toString().padStart(2, "0");
    const seconds = date.getSeconds().toString().padStart(2, "0");
    const milliseconds = date.getMilliseconds().toString().padStart(3, "0");

    return `${day}.${month}.${year} ${hours}:${minutes}:${seconds}.${milliseconds}`;
  }

  // -----------------
  // Race formatters
  // -----------------

  /**
   * Returns a formatted label for the race combining shortLabel and comment.
   *
   * @param race - Race with a short label and optional comment.
   * @returns The formatted race label, or empty string if race is undefined
   * @example
   * ```typescript
   * Formatter.raceLabel({ shortLabel: "M1x", comment: "Final" }); // "M1x Final"
   * Formatter.raceLabel({ shortLabel: "W2x" }); // "W2x"
   * Formatter.raceLabel(undefined); // ""
   * ```
   */
  static raceLabel(race?: Race): string {
    if (!race?.shortLabel) {
      return "";
    }

    let label: string = race.shortLabel;
    if (race.comment) {
      label += ` ${race.comment}`;
    }
    return label;
  }

  /**
   * Returns a formatted label for the race combining race number and race label.
   * The format is "{race.number} - {raceLabel(race)}" where raceLabel includes shortLabel and optional comment.
   *
   * @param race - Race with a number, short label, and optional comment.
   * @returns The formatted race label with number prefix, or empty string if race is undefined
   * @example
   * ```typescript
   * Formatter.nrRaceLabel({ number: 123, shortLabel: "M1x", comment: "Final" }); // "123 - M1x Final"
   * Formatter.nrRaceLabel({ number: 456, shortLabel: "W2x" }); // "456 - W2x"
   * Formatter.nrRaceLabel(undefined); // ""
   * ```
   */
  static nrRaceLabel(race?: Race): string {
    if (!race?.number) {
      return "";
    }

    const raceLabel: string = Formatter.raceLabel(race);
    return raceLabel ? `${race.number} - ${raceLabel}` : "";
  }

  /**
   * Returns the appropriate indication color for race state highlighting.
   *
   * @param race - Race whose cancellation flag and state determine its color.
   * @returns The indication color or undefined if no highlighting needed
   */
  static raceStateHighlight(race?: Race): IndicationColor | undefined {
    if (!race) {
      return undefined;
    }

    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (race.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    }

    switch (race.state) {
      case RaceState.Initial:
      case RaceState.Scheduled:
        return IndicationColor.Indication05; // scheduled -> blue
      case RaceState.Started:
      case RaceState.Unknown:
      case RaceState.Finished:
      case RaceState.PhotoFinish:
        return IndicationColor.Indication03; // started -> orange
      case RaceState.Official:
        return IndicationColor.Indication04; // official -> green
      default:
        return undefined;
    }
  }

  /**
   * Returns a localized label for the race state.
   *
   * @param race - Race whose cancellation flag takes precedence over its state.
   * @returns The localized race state label
   */
  static raceStateLabel(race?: Race): string {
    if (!race) {
      return "";
    }

    if (race.cancelled) {
      return Formatter.i18n("heat.state.cancelled");
    }

    switch (race.state) {
      case RaceState.Initial:
      case RaceState.Scheduled:
        return Formatter.i18n("common.scheduled");
      case RaceState.Started:
      case RaceState.Unknown:
      case RaceState.Finished:
      case RaceState.PhotoFinish:
        return Formatter.i18n("heat.state.started");
      case RaceState.Official:
        return Formatter.i18n("common.finished");
      default:
        return Formatter.i18n("common.scheduled");
    }
  }

  /**
   * Formats the race distance with unit.
   *
   * @param race - Race with a distance in meters.
   * @returns The formatted distance string with "m" suffix
   */
  static distanceLabel(race?: Race): string {
    return race?.distance ? `${race.distance}m` : "";
  }

  /**
   * Returns the appropriate indication color for race entry highlighting.
   *
   * @param race - Race whose cancellation also marks its entries as cancelled.
   * @param entry - Entry to highlight.
   * @returns The indication color for the entry
   */
  static raceEntryHighlight(race?: Race, entry?: Entry): IndicationColor {
    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (entry?.cancelled || race?.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    }
    return IndicationColor.Indication04; // official -> green
  }

  // -----------------
  // Heat formatters
  // -----------------

  /**
   * Formats noncancelled heats as comma-separated schedule and heat labels.
   *
   * @param race - Race providing the heat count and grouping mode.
   * @param heats - Heats to display, excluding cancelled ones.
   * @returns Joined labels, or the localized "none" label when the race has no heat count or the list is empty.
   */
  static heatsLabel(race?: Race, heats?: Heat[]): string {
    if (!race?.heatsCount || !heats?.length) {
      return Formatter.i18n("sorting.none");
    }

    return heats
      .filter((heat: Heat) => !heat.cancelled)
      .map((heat: Heat) => {
        let label: string = `${Formatter.dayTimeIsoLabel(heat.dateTime)} ${Formatter.heatLabel(heat)}`;
        if (race.groupMode && race.groupMode > 0 && heat.groupValue !== undefined) {
          label += ` ${Formatter.groupValueLabel(heat.groupValue)}`;
        }
        return label;
      })
      .join(", ");
  }

  /**
   * Returns the appropriate indication color for heat entry highlighting.
   *
   * @param heatEntry - Entry with an optional result; cancellation takes precedence over result highlighting.
   * @returns The indication color for the heat entry
   */
  static heatEntryHighlight(heatEntry?: HeatEntry): IndicationColor {
    if (!heatEntry) {
      return IndicationColor.Indication05; // default -> blue
    }

    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (heatEntry.entry.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    }

    if (!heatEntry.result) {
      return IndicationColor.Indication05; // no result yet -> blue
    }

    const rankSort: number | undefined = heatEntry.result.rankSort;
    if (rankSort && rankSort > 0 && rankSort <= 5) {
      return IndicationColor.Indication04; // official -> green
    }

    return IndicationColor.Indication02; // DNS, DNF, ... -> red
  }

  /**
   * Formats an athlete label with name, year, and club information.
   *
   * @param athlete - Athlete with birth year and club information.
   * @returns Formatted athlete label
   */
  static athleteLabel(athlete?: Athlete): string {
    if (!athlete) {
      return "";
    }

    let label: string = `${athlete.firstName} ${athlete.lastName} (${athlete.year}`;

    const club: Club = athlete.club;
    if (club.abbreviation) {
      label += `, ${club.abbreviation}`;
    } else if (club.shortName) {
      label += `, ${club.shortName}`;
    } else if (club.longName) {
      label += `, ${club.longName}`;
    }

    return `${label})`;
  }

  /**
   * Formats a crew label with position and athlete information.
   *
   * @param crews - Crew members in display order; coxswains use `St` as their position label.
   * @returns Formatted crew label
   */
  static crewLabel(crews?: Crew[]): string {
    if (!crews?.length) {
      return "";
    }

    const crewLabels: string[] = crews.map((crew: Crew) => {
      const position: string = crew.cox ? "St" : crew.pos?.toString() || "";
      return `${position}: ${Formatter.athleteLabel(crew.athlete)}`;
    });

    return crewLabels.join(", ");
  }

  /**
   * Formats a boat label based on group mode.
   *
   * @param groupMode - Mode 2 includes age-group and comment information.
   * @param entry - Entry providing the label and optional boat number.
   * @returns Formatted boat label
   */
  static boatLabel(groupMode: number, entry?: Entry): string {
    if (!entry?.shortLabel) {
      return "";
    }

    let label: string = entry.shortLabel;

    if (groupMode === 2) {
      if (entry.groupValue !== undefined) {
        label += ` - ${Formatter.groupValueLabel(entry.groupValue)}`;
      }
      if (entry.comment) {
        label += `  (${entry.comment})`;
      }
    }
    if (entry.boatNumber) {
      label += ` - Boot ${entry.boatNumber}`;
    }

    return label;
  }

  /**
   * Formats a boat label with bib number prefix.
   *
   * @param groupMode - Grouping mode passed to {@link boatLabel}.
   * @param entry - Entry providing the optional bib prefix.
   * @returns Formatted boat label with bib number
   */
  static bibBoatLabel(groupMode: number, entry?: Entry): string {
    const boatLabel: string = Formatter.boatLabel(groupMode, entry);
    return entry?.bib ? `${entry.bib} - ${boatLabel}` : boatLabel;
  }

  /**
   * Formats a date with weekday prefix.
   *
   * @param date - ISO date string; the weekday uses the browser's local date interpretation.
   * @returns Formatted weekday and date string
   */
  static weekDayDateLabel(date?: string): string {
    if (!date) {
      return "";
    }

    const weekday: string = Formatter.weekdayLabel(new Date(date).getDay());
    const dateLabel: string = Formatter.dateLabel(date);
    return `${weekday}, ${dateLabel}`;
  }

  /**
   * Formats a date-time string using a German weekday abbreviation and UTC hours/minutes.
   * The weekday is derived in the browser's local time zone, unlike the time portion.
   *
   * @param dateTime - ISO date-time string.
   * @returns Formatted weekday and time string
   */
  static dayTimeIsoLabel(dateTime?: string): string {
    if (!dateTime) {
      return "";
    }

    const oDateTime: Date = new Date(dateTime);
    const weekday: string = Formatter.weekdayLabel(oDateTime.getDay());
    const hours: string = oDateTime.getUTCHours().toString().padStart(2, "0");
    const minutes: string = oDateTime.getUTCMinutes().toString().padStart(2, "0");
    return `${weekday}, ${hours}:${minutes}`;
  }

  /**
   * Formats a time string to HH:MM format.
   *
   * @param time - Colon-separated time string; seconds and subsequent components are omitted.
   * @returns Formatted time string
   */
  static timeLabel(time?: string): string {
    if (!time) {
      return "";
    }

    const timeParts: string[] = time.split(":");
    return timeParts.length >= 2 ? `${timeParts[0]}:${timeParts[1]}` : "";
  }

  /**
   * Returns a localized label for the heat state.
   *
   * @param heat - Heat whose cancellation flag takes precedence over its state code.
   * @returns The localized state label, or `undefined` when no heat is supplied.
   */
  static heatStateLabel(heat?: Heat): string | undefined {
    if (!heat) {
      return undefined;
    }

    if (heat.cancelled) {
      return Formatter.i18n("heat.state.cancelled");
    }

    return Formatter.heatStateLabel2(heat.state);
  }

  /**
   * Maps a raw heat-state code to its localized label, defaulting to scheduled for missing or unknown codes.
   *
   * @param heatState - Optional Aquarius numeric heat-state code.
   * @returns The localized state label, falling back to the scheduled label for missing or unknown codes.
   */
  static heatStateLabel2(heatState?: number): string | undefined {
    switch (heatState) {
      case HeatState.Initial:
        return Formatter.i18n("common.scheduled");
      case HeatState.Seeded:
        return Formatter.i18n("common.seeded");
      case HeatState.Started:
        return Formatter.i18n("heat.state.started");
      case HeatState.Official:
        return Formatter.i18n("heat.state.official");
      case HeatState.Finished:
        return Formatter.i18n("heat.state.finished");
      case HeatState.PhotoFinish:
        return Formatter.i18n("heat.state.photoFinish");
      default:
        return Formatter.i18n("common.scheduled");
    }
  }

  /**
   * Returns the appropriate indication color for heat state highlighting.
   *
   * @param heat - Heat whose cancellation flag and state determine its color.
   * @returns The indication color or undefined if no highlighting needed
   */
  static heatStateHighlight(heat?: Heat): IndicationColor | undefined {
    if (!heat) {
      return undefined;
    }

    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (heat.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    }

    switch (heat.state) {
      default:
      case HeatState.Initial:
        return undefined; // initial -> no color
      case HeatState.Seeded:
        return IndicationColor.Indication05; // scheduled -> blue
      case HeatState.Started:
        return IndicationColor.Indication03; // started -> orange
      case HeatState.Official:
        return IndicationColor.Indication04; // official -> green
      case HeatState.Finished:
        return IndicationColor.Indication06; // finished -> dark green
      case HeatState.PhotoFinish:
        return IndicationColor.Indication07; // photo finish -> custom color
    }
  }

  /**
   * Formats a heat label with group value and round information.
   *
   * @param heat - Heat providing optional age-subclass, round, and heat-number information.
   * @returns Formatted heat label
   */
  static heatLabel(heat?: Heat): string {
    if (!heat) {
      return "";
    }

    const parts: string[] = [];

    // Add group value if applicable
    if (heat.race?.ageClass?.numSubClasses && heat.race.ageClass.numSubClasses > 0 && heat.groupValue !== undefined) {
      parts.push(Formatter.groupValueLabel(heat.groupValue));
    }

    // Add round label if applicable
    const roundLabel: string | undefined = heat.roundCode ? Formatter.roundLabel(heat.roundCode) : undefined;
    if (roundLabel) {
      const heatNumber: string = heat.label || "";
      parts.push(heatNumber ? `${roundLabel} ${heatNumber}` : roundLabel);
    } else if (heat.label) {
      parts.push(heat.label);
    }

    return parts.join(" ");
  }

  /**
   * Returns a localized label for the round code.
   *
   * @param roundCode - Aquarius round-code identifier.
   * @returns The localized round label or undefined if not found
   * @example
   * ```typescript
   * Formatter.roundLabel("F"); // Localized final label.
   * Formatter.roundLabel("S"); // Localized semifinal label.
   * Formatter.roundLabel(undefined); // undefined
   * ```
   */
  static roundLabel(roundCode?: RoundCode): string | undefined {
    if (!roundCode) {
      return undefined;
    }

    switch (roundCode) {
      case "A":
        return Formatter.i18n("heat.label.division");
      case "H":
        return Formatter.i18n("heat.label.repechage");
      case "R":
        return Formatter.i18n("heat.label.mainRace");
      case "V":
        return Formatter.i18n("heat.label.forerun");
      case "S":
        return Formatter.i18n("heat.label.semifinal");
      case "F":
        return Formatter.i18n("heat.label.final");
      default:
        return undefined;
    }
  }

  // -----------------
  // Private helper methods
  // -----------------

  /**
   * Maps a group value number to its corresponding label.
   *
   * @param groupValue - Encoded age-group value from {@link GroupValue}.
   * @returns The formatted group label (e.g., "AK A", "AK B")
   */
  private static groupValueLabel(groupValue: number): string {
    const prefix: string = "AK ";

    switch (groupValue) {
      case GroupValue.A:
        return `${prefix}A`;
      case GroupValue.B:
        return `${prefix}B`;
      case GroupValue.C:
        return `${prefix}C`;
      case GroupValue.D:
        return `${prefix}D`;
      case GroupValue.E:
        return `${prefix}E`;
      case GroupValue.F:
        return `${prefix}F`;
      case GroupValue.G:
        return `${prefix}G`;
      case GroupValue.H:
        return `${prefix}H`;
      case GroupValue.I:
        return `${prefix}I`;
      case GroupValue.J:
        return `${prefix}J`;
      default:
        return "";
    }
  }

  /**
   * Reorders hyphen-separated date components from `YYYY-MM-DD` to `DD.MM.YYYY` without validating the date.
   *
   * @param date - Date-only string in ISO component order.
   * @returns Reordered components, or empty text when the input is missing or has fewer than three components.
   */
  private static dateLabel(date?: string): string {
    if (!date) {
      return "";
    }

    const dateParts: string[] = date.split("-");
    if (dateParts.length >= 3) {
      return `${dateParts[2]}.${dateParts[1]}.${dateParts[0]}`;
    }
    return "";
  }

  /**
   * Returns a short German label for the weekday.
   *
   * @param weekday - Weekday number: 0 = Sunday through 6 = Saturday; 7 also maps to Sunday.
   * @returns The short German label for the weekday
   */
  private static weekdayLabel(weekday: number): string {
    switch (weekday) {
      case 0:
      case 7:
        return "So";
      case 1:
        return "Mo";
      case 2:
        return "Di";
      case 3:
        return "Mi";
      case 4:
        return "Do";
      case 5:
        return "Fr";
      case 6:
        return "Sa";
      default:
        return "";
    }
  }

  /**
   * Gets a localized text from the resource bundle.
   *
   * If the bundle has not been initialised yet (e.g., a formatter is invoked
   * before bundle resolution), returns the i18n key itself rather than throwing.
   *
   * @param key - Resource-bundle key.
   * @param args - Optional values substituted into translation placeholders.
   * @returns Localized text, or the key if the bundle or translated text is unavailable.
   */
  private static i18n(key: string, args?: any[]): string {
    if (!Formatter.bundle) {
      Log.warning(`Formatter.i18n("${key}") called before Formatter.init() — returning key as-is.`);
      return key;
    }
    return Formatter.bundle.getText(key, args) || key;
  }
}
