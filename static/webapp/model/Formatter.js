sap.ui.define([
  "sap/ui/core/IndicationColor"
], function (IndicationColor) {
  "use strict";

  const Formatter = {

    // -----------------
    // race formatters
    // -----------------
    raceLabel: function (oRace) {
      if (oRace) {
        let label = oRace.shortLabel;
        if (oRace.comment) {
          label += " " + oRace.comment;
        }
        return label;
      }
      return "";
    },

    nrRaceLabel: function (oRace) {
      if (oRace) {
        const label = oRace.number + " - " + Formatter.raceLabel(oRace);
        return label;
      }
      return "";
    },

    raceStateHighlight: function (oRace) {
      if (!oRace) {
        return "";
      }
      // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
      if (oRace.cancelled) {
        return IndicationColor.Indication02; // cancelled -> red
      } else {
        switch (oRace.state) {
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
    },

    raceStateLabel: function (oRace) {
      if (!oRace) {
        return "";
      }
      if (oRace.cancelled) {
        return this.i18n("heat.state.cancelled");
      } else {
        switch (oRace.state) {
          default:
          case 0:
          case 1:
            return this.i18n("common.scheduled");
          case 2:
          case 3:
          case 5:
          case 6:
            return this.i18n("heat.state.started");
          case 4:
            return this.i18n("common.finished");
        }
      }
    },

    distanceLabel: function (oRace) {
      if (oRace?.distance) {
        return oRace.distance + "m";
      }
      return "";
    },

    raceRegistrationHighlight: function (oRegistration) {
      // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
      if (oRegistration.cancelled || oRegistration.race.cancelled) {
        return IndicationColor.Indication02; // cancelled -> red
      } else {
        return IndicationColor.Indication04; // official -> green
      }
    },

    // -----------------
    // heat formatters
    // -----------------
    heatRegistrationHighlight: function (oHeatRegistration) {
      // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
      if (oHeatRegistration.registration.cancelled) {
        return IndicationColor.Indication02; // cancelled -> red
      } else
        if (!oHeatRegistration.result) {
          return IndicationColor.Indication05; // no result yet -> blue
        } else
          if (oHeatRegistration.result.rankSort > 0 && oHeatRegistration.result.rankSort <= 4) {
            return IndicationColor.Indication04; // official -> green
          } else {
            return IndicationColor.Indication02; // DNS, DNF, ... -> red
          }
    },

    athleteLabel: function (oAthlete) {
      let sLabel = oAthlete.firstName + " " + oAthlete.lastName + " (" + oAthlete.year + ", ";
      if (oAthlete.club.abbreviation) {
        sLabel += oAthlete.club.abbreviation
      } else {
        sLabel += oAthlete.club.shortName
      };
      sLabel += ")"
      return sLabel;
    },

    crewLabel: function (aCrew) {
      let label = "";
      if (aCrew) {
        for (const oCrew of aCrew) {
          label += (oCrew.cox ? "St" : oCrew.pos) + ": " + Formatter.athleteLabel(oCrew.athlete) + ", ";
        }
        label = label.substring(0, label.length - 2);
      }
      return label;
    },

    boatLabel: function (oRegistration) {
      let sLabel = "" + oRegistration.shortLabel;
      if (oRegistration.race.groupMode == 2) {
        sLabel += " - " + Formatter.groupValueLabel(oRegistration.groupValue);
      }
      if (oRegistration.boatNumber) {
        sLabel += " - Boot " + oRegistration.boatNumber;
      }
      if (oRegistration.comment) {
        sLabel += "  (" + oRegistration.comment + ")";
      }
      return sLabel;
    },

    groupValueLabel: function (iGroupValue) {
      const PREFIX = "AK ";
      switch (iGroupValue) {
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
    },

    weekdayLabel: function (iWeekday) {
      switch (iWeekday) {
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
    },

    weekDayDateLabel: function (sDate) {
      if (sDate) {
        const sWeekday = Formatter.weekdayLabel(new Date(sDate).getDay());
        const sDateLabel = Formatter.dateLabel(sDate);
        return `${sWeekday}, ${sDateLabel}`;
      }
      return "";
    },

    dayTimeIsoLabel: function (sDateTime) {
      if (sDateTime) {
        const oDateTime = new Date(sDateTime);
        const sWeekday = Formatter.weekdayLabel(oDateTime.getDay());
        const sHours = ('00' + oDateTime.getUTCHours()).slice(-2);
        const sMinutes = ('00' + oDateTime.getUTCMinutes()).slice(-2);
        return sWeekday + ", " + sHours + ":" + sMinutes;
      }
      return "";
    },

    timeLabel: function (sDate) {
      if (sDate) {
        const aDate = sDate.split(":");
        return aDate[0] + ":" + aDate[1];
      }
      return "";
    },

    dateLabel: function (sDate) {
      if (sDate) {
        const aDate = sDate.split("-");
        return aDate[2] + "." + aDate[1] + "." + aDate[0];
      }
      return "";
    },

    heatStateLabel: function (oHeat) {
      if (!oHeat) {
        return "";
      }
      if (oHeat.cancelled) {
        return this.i18n("heat.state.cancelled");
      } else {
        switch (oHeat.state) {
          default:
          case 0:
            return this.i18n("common.scheduled");
          case 1:
            return this.i18n("common.seeded");
          case 2:
            return this.i18n("heat.state.started");
          case 4:
            return this.i18n("heat.state.official");
          case 5:
            return this.i18n("heat.state.finished");
          case 6:
            return this.i18n("heat.state.photoFinish");
        }
      }
    },

    heatStateHighlight: function (oHeat) {
      if (!oHeat) {
        return "";
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
    },

    heatLabel: function (oHeat) {
      if (!oHeat) {
        return "";
      }

      let sGroupValue = "";
      if (oHeat.race && oHeat.race.ageClass.numSubClasses > 0) {
        sGroupValue = " - " + Formatter.groupValueLabel(oHeat.groupValue);
      }
      const sHeatLabel = oHeat.label || "";

      switch (oHeat.roundCode) {
        case "A":
          return this.i18n("heat.label.division", [sHeatLabel, sGroupValue]);
        case "R":
          return this.i18n("heat.label.mainRace", [sHeatLabel, sGroupValue]);
        case "V":
          return this.i18n("heat.label.forerun", [sHeatLabel, sGroupValue]);
        case "F":
          return this.i18n("heat.label.final", [sGroupValue]);
        default:
          return "";
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
