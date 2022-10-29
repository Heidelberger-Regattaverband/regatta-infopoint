sap.ui.define(function () {
  "use strict";

  let Formatter = {

    distanceLabel: function (oRace) {
      if (oRace) {
        return oRace.distance + "m";
      }
      return "";
    },

    boatLabel: function (sShortLabel, iBoatNumber, sComment) {
      if (iBoatNumber > 0) {
        return sShortLabel + " - Boot " + iBoatNumber;
      }
      if (sComment) {
        sShortLabel += "  (" + sComment + ")";
      }
      return sShortLabel;
    },

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
        let label = oRace.number + " - " + oRace.shortLabel;
        if (oRace.comment) {
          label += " " + oRace.comment;
        }
        return label;
      }
      return "";
    },

    dayLabel: function (sDate) {
      if (sDate) {
        const aDate = sDate.split("-");
        return aDate[2] + "." + aDate[1] + ".";
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

    stateLabel: function (oHeat) {
      const resourceBundle = this.getView().getModel("i18n").getResourceBundle();

      if (oHeat.cancelled) {
        return resourceBundle.getText("heat.state.cancelled");
      } else {
        switch (oHeat.state) {
          default:
          case 0:
            return resourceBundle.getText("heat.state.initial");
          case 1:
            return resourceBundle.getText("heat.state.scheduled");
          case 2:
            return resourceBundle.getText("heat.state.started");
          case 4:
            return resourceBundle.getText("heat.state.official");
          case 5:
            return resourceBundle.getText("heat.state.finished");
          case 6:
            return resourceBundle.getText("heat.state.photoFinish");
        }
      }
    },

    heatLabel: function (oHeat) {
      let sGroupValue = "";

      if (oHeat.ac_num_sub_classes > 0) {
        const PREFIX = " - AK ";
        switch (oHeat.group_value) {
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

      const oResourceBundle = this.getView().getModel("i18n").getResourceBundle();
      switch (oHeat.round_code) {
        case "A":
          return oResourceBundle.getText("heat.label.division", [oHeat.label, sGroupValue]);
        case "R":
          return oResourceBundle.getText("heat.label.mainRace", [oHeat.label, sGroupValue]);
        case "V":
          return oResourceBundle.getText("heat.label.forerun", [oHeat.label, sGroupValue]);
        case "F":
          return oResourceBundle.getText("heat.label.final", [sGroupValue]);
        default:
          return "";
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
