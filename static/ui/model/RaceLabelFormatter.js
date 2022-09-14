sap.ui.define(function () {
  "use strict";

  var Formatter = {
    raceLabel: function (sRaceNumber, sRoundCode, sRoundLabel, iNumSubClasses, iGroupValue) {
      var sGroupValue = "";
      if (iNumSubClasses > 0) {
        switch (iGroupValue) {
          case 0:
            sGroupValue = " - AK A";
            break;
          case 4:
            sGroupValue = " - AK B";
            break;
          case 8:
            sGroupValue = " - AK C";
            break;
          case 12:
            sGroupValue = " - AK D";
            break;
          case 16:
            sGroupValue = " - AK E";
            break;
          case 20:
            sGroupValue = " - AK F";
            break;
          case 24:
            sGroupValue = " - AK G";
            break;
          case 28:
            sGroupValue = " - AK H";
            break;
          case 32:
            sGroupValue = " - AK I";
            break;
          case 36:
            sGroupValue = " - AK J";
            break;
        }
      }

      switch (sRoundCode) {
        case "A":
          return sRaceNumber + " - Abteilung " + sRoundLabel;
        case "R":
          return sRaceNumber + " - Hauptrennen" + sGroupValue;
        case "V":
          return sRaceNumber + " - Vorlauf " + sRoundLabel;
        case "F":
          return sRaceNumber + " - Finale";
        default:
          return sRaceNumber;
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
