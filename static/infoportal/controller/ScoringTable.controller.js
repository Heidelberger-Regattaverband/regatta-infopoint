sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller"
], function (BaseController) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("scoring").attachMatched(this._loadScoringModel, this);

      this.scoringTable = this.getView().byId("racesTable");
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    _loadScoringModel: async function () {
      const oScoringModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/scoring", this.scoringTable);
      this.setViewModel(oScoringModel, "scoring");
    }

  });
});