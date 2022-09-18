sap.ui.define(function () {
  "use strict";

  var Formatter = {
    heatLabel: function (sRoundCode, sRoundLabel, iNumSubClasses, iGroupValue) {
      var sGroupValue = "";
      if (iNumSubClasses > 0) {
        const PREFIX = " - AK ";
        switch (iGroupValue) {
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

      switch (sRoundCode) {
        case "A":
          return "Abteilung " + sRoundLabel + sGroupValue;
        case "R":
          return "Hauptrennen" + sGroupValue;
        case "V":
          return "Vorlauf " + sRoundLabel + sGroupValue;
        case "F":
          return "Finale" + sGroupValue;
        default:
          return "";
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
