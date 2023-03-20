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
      if (oRace && oRace.distance) {
        return oRace.distance + "m";
      }
      return "";
    },

    raceRegistrationHighlight: function (oRegistration) {
      // https://experience.sap.com/fiori-design-web/quartz-light-colors/#indication-colors
      if (oRegistration.cancelled) {
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

    crewLabel: function (aCrew) {
      let label = "";
      if (aCrew) {
        for (const oCrew of aCrew) {
          const athlete = oCrew.athlete;
          label += (oCrew.cox ? "St" : oCrew.pos) + ": " + athlete.firstName + " " + athlete.lastName + " (" + athlete.year + ", " + athlete.club + "), ";
        }
        label = label.substring(0, label.length - 2);
      }
      return label;
    },

    boatLabel: function (oRegistration) {
      let sLabel = "" + oRegistration.shortLabel;
      if (oRegistration.boatNumber > 0) {
        sLabel += " - Boot " + oRegistration.boatNumber;
      }
      if (oRegistration.comment) {
        sLabel += "  (" + oRegistration.comment + ")";
      }
      return sLabel;
    },

    weekdayLabel: function (iWeekday) {
      switch (iWeekday) {
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

    dayTimeLabel: function (oHeat) {
      if (oHeat) {
        const sWeekday = Formatter.weekdayLabel(oHeat.weekday);
        const aTime = oHeat.time.split(":");
        return sWeekday + ", " + aTime[0] + ":" + aTime[1];
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
        const PREFIX = " - AK ";
        switch (oHeat.groupValue) {
          case 0:
            sGroupValue = PREFIX + "A";
            break;
          case 4:
            sGroupValue = PREFIX + "B";
            break;
          case 8:
            sGroupValue = PREFIX + "C";
            break;
          case 12:
            sGroupValue = PREFIX + "D";
            break;
          case 16:
            sGroupValue = PREFIX + "E";
            break;
          case 20:
            sGroupValue = PREFIX + "F";
            break;
          case 24:
            sGroupValue = PREFIX + "G";
            break;
          case 28:
            sGroupValue = PREFIX + "H";
            break;
          case 32:
            sGroupValue = PREFIX + "I";
            break;
          case 36:
            sGroupValue = PREFIX + "J";
            break;
        }
      }

      switch (oHeat.roundCode) {
        case "A":
          return this.i18n("heat.label.division", [oHeat.label, sGroupValue]);
        case "R":
          return this.i18n("heat.label.mainRace", [oHeat.label, sGroupValue]);
        case "V":
          return this.i18n("heat.label.forerun", [oHeat.label, sGroupValue]);
        case "F":
          return this.i18n("heat.label.final", [sGroupValue]);
        default:
          return "";
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
