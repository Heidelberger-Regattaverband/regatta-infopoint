sap.ui.define(function () {
  "use strict";

  var Formatter = {
    format: function (sDate) {
      const aDate = sDate.split(":");
      return aDate[0] + ":" + aDate[1];
    }
  };

  return Formatter;
}, /* bExport= */ true);
