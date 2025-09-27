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
  GroupValue,
} from "./types";

/**
 * Formatter utility class for regatta application data formatting.
 * Provides static methods for formatting various data types used in the regatta application.
 * 
 * @namespace de.regatta_hd.infoportal.model
 */
export default class Formatter {
  private static bundle: ResourceBundle;

  static {
    Formatter.bundle = ResourceBundle.create({
      // specify url of the base .properties file
      url: "i18n/i18n.properties",
      async: false,
      supportedLocales: ["de", "en"],
      fallbackLocale: "de",
    }) as ResourceBundle;
  }

  // -----------------
  // Race formatters
  // -----------------

  /**
   * Returns a formatted label for the race combining shortLabel and comment.
   * @param race The race object with shortLabel and optional comment properties
   * @returns The formatted race label, or empty string if race is undefined
   * @example
   * ```typescript
   * raceLabel({shortLabel: "M1x", comment: "Final"}) // returns "M1x Final"
   * raceLabel({shortLabel: "W2x"}) // returns "W2x"
   * raceLabel(undefined) // returns ""
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
   * @param race The race object with number, shortLabel and optional comment properties
   * @returns The formatted race label with number prefix, or empty string if race is undefined
   * @example
   * ```typescript
   * nrRaceLabel({number: 123, shortLabel: "M1x", comment: "Final"}) // returns "123 - M1x Final"
   * nrRaceLabel({number: 456, shortLabel: "W2x"}) // returns "456 - W2x"
   * nrRaceLabel(undefined) // returns ""
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
   * @param race The race object
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
      case RaceState.Official: return IndicationColor.Indication04; // official -> green
      default: return undefined;
    }
  }

  /**
   * Returns a localized label for the race state.
   * @param race The race object
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
   * @param race The race object
   * @returns The formatted distance string with "m" suffix
   */
  static distanceLabel(race?: Race): string {
    return race?.distance ? `${race.distance}m` : "";
  }

  /**
   * Returns the appropriate indication color for race entry highlighting.
   * @param race The race object
   * @param entry The entry object
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
   * Formats a list of heats for display.
   * @param race The race object
   * @param heats Array of heat objects
   * @returns Formatted string of heat labels
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
   * @param heatEntry The heat entry object
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
   * @param athlete The athlete object
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
   * @param crews Array of crew members
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
   * @param groupMode The group mode setting
   * @param entry The entry object
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
      if (entry.boatNumber) {
        label += ` - Boot ${entry.boatNumber}`;
      }
      if (entry.comment) {
        label += `  (${entry.comment})`;
      }
    }

    return label;
  }

  /**
   * Formats a boat label with bib number prefix.
   * @param groupMode The group mode setting
   * @param entry The entry object
   * @returns Formatted boat label with bib number
   */
  static bibBoatLabel(groupMode: number, entry?: Entry): string {
    const boatLabel: string = Formatter.boatLabel(groupMode, entry);
    return entry?.bib ? `${entry.bib} - ${boatLabel}` : boatLabel;
  }

  /**
   * Formats a date with weekday prefix.
   * @param date The ISO date string
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
   * Formats a datetime string with weekday and time.
   * @param dateTime The ISO datetime string
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
   * @param time The time string
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
   * @param heat The heat object
   * @returns The localized heat state label
   */
  static heatStateLabel(heat?: Heat): string | undefined {
    if (!heat) {
      return undefined;
    }

    if (heat.cancelled) {
      return Formatter.i18n("heat.state.cancelled");
    }

    switch (heat.state) {
      case HeatState.Initial: return Formatter.i18n("common.scheduled");
      case HeatState.Seeded: return Formatter.i18n("common.seeded");
      case HeatState.Started: return Formatter.i18n("heat.state.started");
      case HeatState.Official: return Formatter.i18n("heat.state.official");
      case HeatState.Finished: return Formatter.i18n("heat.state.finished");
      case HeatState.PhotoFinish: return Formatter.i18n("heat.state.photoFinish");
      default: return Formatter.i18n("common.scheduled");
    }
  }

  /**
   * Returns the appropriate indication color for heat state highlighting.
   * @param heat The heat object
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
      case HeatState.Initial: return undefined; // initial -> no color
      case HeatState.Seeded: return IndicationColor.Indication05; // scheduled -> blue
      case HeatState.Started: return IndicationColor.Indication03; // started -> orange
      case HeatState.Official: return IndicationColor.Indication04; // official -> green
      case HeatState.Finished: return IndicationColor.Indication06; // finished -> dark green
      case HeatState.PhotoFinish: return IndicationColor.Indication07; // photo finish -> custom color
    }
  }

  /**
   * Formats a heat label with group value and round information.
   * @param heat The heat object
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
   * @param roundCode The round code identifier
   * @returns The localized round label or undefined if not found
   * @example
   * ```typescript
   * roundLabel("F") // returns "Final"
   * roundLabel("S") // returns "Semifinal"
   * roundLabel("X") // returns undefined
   * ```
   */
  static roundLabel(roundCode?: RoundCode): string | undefined {
    if (!roundCode) {
      return undefined;
    }

    switch (roundCode) {
      case "A": return Formatter.i18n("heat.label.division");
      case "H": return Formatter.i18n("heat.label.repechage");
      case "R": return Formatter.i18n("heat.label.mainRace");
      case "V": return Formatter.i18n("heat.label.forerun");
      case "S": return Formatter.i18n("heat.label.semifinal");
      case "F": return Formatter.i18n("heat.label.final");
      default: return undefined;
    }
  }

  // -----------------
  // Private helper methods
  // -----------------

  /**
   * Maps a group value number to its corresponding label.
   * @param groupValue The numeric group value
   * @returns The formatted group label (e.g., "AK A", "AK B")
   */
  private static groupValueLabel(groupValue: number): string {
    const prefix: string = "AK ";

    switch (groupValue) {
      case GroupValue.A: return `${prefix}A`;
      case GroupValue.B: return `${prefix}B`;
      case GroupValue.C: return `${prefix}C`;
      case GroupValue.D: return `${prefix}D`;
      case GroupValue.E: return `${prefix}E`;
      case GroupValue.F: return `${prefix}F`;
      case GroupValue.G: return `${prefix}G`;
      case GroupValue.H: return `${prefix}H`;
      case GroupValue.I: return `${prefix}I`;
      case GroupValue.J: return `${prefix}J`;
      default: return "";
    }
  }

  /**
   * Formats a date string from ISO format (YYYY-MM-DD) to German date format (DD.MM.YYYY).
   * @param date The date string in ISO format (YYYY-MM-DD)
   * @returns The formatted date string in German format (DD.MM.YYYY), or empty string if input is invalid
   * @example
   * ```typescript
   * dateLabel("2024-03-15") // returns "15.03.2024"
   * dateLabel("2023-12-31") // returns "31.12.2023"
   * dateLabel(undefined)    // returns ""
   * dateLabel("invalid")    // returns ""
   * ```
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
   * @param weekday The weekday number: 0 = Sunday, 1 = Monday, ..., 6 = Saturday, 7 = Sunday
   * @returns The short German label for the weekday
   */
  private static weekdayLabel(weekday: number): string {
    switch (weekday) {
      case 0:
      case 7: return "So";
      case 1: return "Mo";
      case 2: return "Di";
      case 3: return "Mi";
      case 4: return "Do";
      case 5: return "Fr";
      case 6: return "Sa";
      default: return "";
    }
  }

  /**
   * Gets a localized text from the resource bundle.
   * @param key The i18n key
   * @param args Optional arguments for text interpolation
   * @returns The localized text or the key if not found
   */
  private static i18n(key: string, args?: any[]): string {
    return Formatter.bundle.getText(key, args) || key;
  }
}
