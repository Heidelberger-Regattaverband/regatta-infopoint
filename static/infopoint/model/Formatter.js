sap.ui.define(function () {
  "use strict";

  var Formatter = {
    boatLabel: function (sShortLabel, iBoatNumber) {
      if (iBoatNumber > 0) {
        return sShortLabel + " - Boot " + iBoatNumber;
      }
      return sShortLabel;
    },

    raceLabel: function (sRaceNumber, sRaceShortLabel, sComment) {
      return sRaceNumber + " - " + sRaceShortLabel + " " + sComment;
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

    stateLabel: function (iState, bCancelled) {
      var resourceBundle = this.getView().getModel("i18n").getResourceBundle();

      if (bCancelled) {
        return resourceBundle.getText("heat.state.cancelled");
      } else {
        switch (iState) {
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
    }

  };

  return Formatter;
}, /* bExport= */ true);
