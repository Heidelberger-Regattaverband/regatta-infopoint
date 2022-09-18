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
      const aDate = sDate.split("-");
      return aDate[2] + "." + aDate[1] + ".";
    },

    timeLabel: function (sDate) {
      const aDate = sDate.split(":");
      return aDate[0] + ":" + aDate[1];
    },

    dateLabel: function (sDate) {
      const aDate = sDate.split("-");
      return aDate[2] + "." + aDate[1] + "." + aDate[0];
    },

  };

  return Formatter;
}, /* bExport= */ true);
