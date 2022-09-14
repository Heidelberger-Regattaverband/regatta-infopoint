sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "../model/StateLabelFormatter",
  "../model/RaceLabelFormatter"
], function (Controller, StateLabelFormatter, RaceLabelFormatter) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {
    stateLabelFormatter: StateLabelFormatter,

    onSelectionChange: function (oEvent) {
      var oSelectedItem = oEvent.getParameter("listItem");
      var sPath = oSelectedItem.getBindingContext("heat").getPath();
      var oItem = this.getView().getModel("heat").getProperty(sPath);
      alert(JSON.stringify(oItem));
    }

  });
});