import ResourceBundle from "sap/base/i18n/ResourceBundle";
import { IndicationColor } from "sap/ui/core/library";

/**
 * @namespace de.regatta_hd.infoportal.controller
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
  // race formatters
  // -----------------
  static raceLabel(race?: any): string {
    if (race) {
      let label: string = race.shortLabel;
      if (race.comment) {
        label += " " + race.comment;
      }
      return label;
    }
    return "";
  }

  static nrRaceLabel(race?: any): string {
    if (race) {
      const label: string = race.number + " - " + Formatter.raceLabel(race);
      return label;
    }
    return "";
  }

  static raceStateHighlight(race: any): IndicationColor | undefined {
    if (!race) {
      return undefined;
    }
    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (race.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    } else {
      switch (race.state) {
        default:
        case 0: // initial
        case 1: // scheduled
          return IndicationColor.Indication05; // scheduled -> blue
        case 2: // started
        case 3: // ???
        case 5: // finished
        case 6: // photo finish
          return IndicationColor.Indication03; // started -> orange
        case 4:
          return IndicationColor.Indication04; // official -> green
      }
    }
  }

  static raceStateLabel(race?: any): string {
    if (!race) {
      return "";
    }
    if (race.cancelled) {
      return Formatter.i18n("heat.state.cancelled");
    } else {
      switch (race.state) {
        default:
        case 0:
        case 1:
          return Formatter.i18n("common.scheduled");
        case 2:
        case 3:
        case 5:
        case 6:
          return Formatter.i18n("heat.state.started");
        case 4:
          return Formatter.i18n("common.finished");
      }
    }
  }

  static distanceLabel(race?: any): string {
    if (race?.distance) {
      return race.distance + "m";
    }
    return "";
  }

  static raceRegistrationHighlight(race: any, registration: any): IndicationColor {
    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (registration.cancelled || race.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    } else {
      return IndicationColor.Indication04; // official -> green
    }
  }

  // -----------------
  // heat formatters
  // -----------------
  static heatsLabel(race: any, heats?: any[]): string {
    if (heats) {
      return heats.filter(heat => !heat.cancelled)
        .map(heat => {
          let label: string = Formatter.dayTimeIsoLabel(heat.dateTime) + " - " + Formatter.heatLabel(heat);
          if (race.groupMode > 0) {
            label += " " + Formatter.groupValueLabel(heat.groupValue);
          }
          return label;
        }).join(", ");
    }
    return Formatter.i18n("sorting.none");
  }

  static heatRegistrationHighlight(heatRegistration: any): IndicationColor {
    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (heatRegistration.registration.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    } else
      if (!heatRegistration.result) {
        return IndicationColor.Indication05; // no result yet -> blue
      } else
        if (heatRegistration.result.rankSort > 0 && heatRegistration.result.rankSort <= 5) {
          return IndicationColor.Indication04; // official -> green
        } else {
          return IndicationColor.Indication02; // DNS, DNF, ... -> red
        }
  }

  static athleteLabel(athlete: any): string {
    let label: string = athlete.firstName + " " + athlete.lastName + " (" + athlete.year;
    if (athlete.club.shortName) {
      label += ", " + athlete.club.shortName
    } else if (athlete.club.abbreviation) {
      label += ", " + athlete.club.abbreviation
    } else if (athlete.club.longName) {
      label += ", " + athlete.club.longName
    };
    label += ")"
    return label;
  }

  static crewLabel(crews?: any[]): string {
    let label: string = "";
    if (crews) {
      for (const crew of crews) {
        label += (crew.cox ? "St" : crew.pos) + ": " + Formatter.athleteLabel(crew.athlete) + ", ";
      }
      label = label.substring(0, label.length - 2);
    }
    return label;
  }

  static boatLabel(registration: any): string {
    let label: string = "" + registration.shortLabel;
    // if (registration.race && registration.race.groupMode == 2) {
    if (registration.groupValue) {
      label += " - " + Formatter.groupValueLabel(registration.groupValue);
    }
    if (registration.boatNumber) {
      label += " - Boot " + registration.boatNumber;
    }
    if (registration.comment) {
      label += "  (" + registration.comment + ")";
    }
    return label;
  }

  static groupValueLabel(groupValue: number): string | undefined {
    const PREFIX: string = "AK ";
    switch (groupValue) {
      case 0:
        return PREFIX + "A";
      case 4:
        return PREFIX + "B";
      case 8:
        return PREFIX + "C";
      case 12:
        return PREFIX + "D";
      case 16:
        return PREFIX + "E";
      case 20:
        return PREFIX + "F";
      case 24:
        return PREFIX + "G";
      case 28:
        return PREFIX + "H";
      case 32:
        return PREFIX + "I";
      case 36:
        return PREFIX + "J";
    }
  }

  private static weekdayLabel(weekday: number): string {
    switch (weekday) {
      case 0: return "So";
      case 1: return "Mo";
      case 2: return "Di";
      case 3: return "Mi";
      case 4: return "Do";
      case 5: return "Fr";
      case 6: return "Sa";
      case 7: return "So";
      default: return "";
    }
  }

  static weekDayDateLabel(date?: string): string {
    if (date) {
      const weekday: string = Formatter.weekdayLabel(new Date(date).getDay());
      const dateLabel: string | undefined = Formatter.dateLabel(date);
      return `${weekday}, ${dateLabel}`;
    }
    return "";
  }

  static dayTimeIsoLabel(dateTime?: string): string {
    if (dateTime) {
      const oDateTime: Date = new Date(dateTime);
      const weekday: string = Formatter.weekdayLabel(oDateTime.getDay());
      const hours: string = ('00' + oDateTime.getUTCHours()).slice(-2);
      const minutes: string = ('00' + oDateTime.getUTCMinutes()).slice(-2);
      return weekday + ", " + hours + ":" + minutes;
    }
    return "";
  }

  static timeLabel(time?: string): string {
    if (time) {
      const timeSplit: string[] = time.split(":");
      return timeSplit[0] + ":" + timeSplit[1];
    }
    return "";
  }

  private static dateLabel(date?: string): string {
    if (date) {
      const dateSplit: string[] = date.split("-");
      return dateSplit[2] + "." + dateSplit[1] + "." + dateSplit[0];
    }
    return "";
  }

  static heatStateLabel(heat?: any): string | undefined {
    if (!heat) {
      return undefined;
    }
    if (heat.cancelled) {
      return Formatter.i18n("heat.state.cancelled");
    } else {
      switch (heat.state) {
        default:
        case 0:
          return Formatter.i18n("common.scheduled");
        case 1:
          return Formatter.i18n("common.seeded");
        case 2:
          return Formatter.i18n("heat.state.started");
        case 4:
          return Formatter.i18n("heat.state.official");
        case 5:
          return Formatter.i18n("heat.state.finished");
        case 6:
          return Formatter.i18n("heat.state.photoFinish");
      }
    }
  }

  static heatStateHighlight(oHeat: any): IndicationColor | undefined {
    if (!oHeat) {
      return undefined; //  -> no color
    }
    // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
    if (oHeat.cancelled) {
      return IndicationColor.Indication02; // cancelled -> red
    } else {
      switch (oHeat.state) {
        default:
        case 0:
          return undefined; // initial -> no color
        case 1:
          return IndicationColor.Indication05; // scheduled -> blue
        case 2:
          return IndicationColor.Indication03; // started -> orange
        case 4:
          return IndicationColor.Indication04; // official -> green
        case 5:
          return IndicationColor.Indication06; // finished -> dark green
        case 6:
          return IndicationColor.Indication07; // photo finish -> ???
      }
    }
  }

  static heatLabel(heat?: any): string {
    if (!heat) {
      return "";
    }

    let groupValue: string = "";
    if (heat.race && heat.race.ageClass && heat.race.ageClass.numSubClasses > 0) {
      groupValue = " - " + Formatter.groupValueLabel(heat.groupValue);
    }
    const heatLabel: string = heat.label || "";

    switch (heat.roundCode) {
      case "A":
        return Formatter.i18n("heat.label.division", [heatLabel, groupValue]);
      case "H":
        return Formatter.i18n("heat.label.repechage", [heatLabel]);
      case "R":
        return Formatter.i18n("heat.label.mainRace", [heatLabel, groupValue]);
      case "V":
        return Formatter.i18n("heat.label.forerun", [heatLabel, groupValue]);
      case "S":
        return Formatter.i18n("heat.label.semifinal", [heatLabel, groupValue]);
      case "F":
        return Formatter.i18n("heat.label.final", [heatLabel, groupValue]);
      default:
        return "";
    }
  }

  static roundLabel(roundCode: string): string {
    switch (roundCode) {
      case "A":
        return "Abteilung";
      case "H":
        return "Hoffnungslauf";
      case "R":
        return "Hauptrennen";
      case "V":
        return "Vorlauf";
      case "S":
        return "Halbfinale";
      case "F":
        return "Finale";
      default:
        return roundCode;
    }
  }

  private static i18n(key: string, args?: any[]): string {
    return Formatter.bundle.getText(key, args) || key;
  }

}
